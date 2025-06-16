use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token, Result, Type, FnArg, Signature,
    parenthesized, braced,
};

pub struct McpResourcesInput {
    pub resources: Vec<ResourceDefinition>,
}

pub struct ResourceDefinition {
    pub name: Ident,
    pub operations: Vec<ResourceOperation>,
}

pub struct ResourceOperation {
    pub op_type: OperationType,
    pub name: Ident,
    pub params: Vec<ResourceParam>,
    pub description: Option<LitStr>,
}

pub struct ResourceParam {
    pub name: Ident,
    pub param_type: Type,
}

pub enum OperationType {
    List,
    Read,
    Write,
}

impl Parse for McpResourcesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut resources = Vec::new();
        
        while !input.is_empty() {
            // Parse 'resource agents { ... }'
            let resource_ident: Ident = input.parse()?;
            if resource_ident != "resource" {
                return Err(syn::Error::new(resource_ident.span(), "Expected 'resource'"));
            }
            
            let name = input.parse::<Ident>()?;
            
            let content;
            braced!(content in input);
            
            let mut operations = Vec::new();
            while !content.is_empty() {
                let op_type_ident = content.parse::<Ident>()?;
                let op_type = match op_type_ident.to_string().as_str() {
                    "list" => OperationType::List,
                    "read" => OperationType::Read,
                    "write" => OperationType::Write,
                    _ => return Err(syn::Error::new(op_type_ident.span(), "Expected 'list', 'read', or 'write'")),
                };
                
                content.parse::<Token![:]>()?;
                let op_name = content.parse::<Ident>()?;
                
                let mut params = Vec::new();
                if content.peek(syn::token::Paren) {
                    let params_content;
                    parenthesized!(params_content in content);
                    
                    while !params_content.is_empty() {
                        let param_name = params_content.parse::<Ident>()?;
                        params_content.parse::<Token![:]>()?;
                        let param_type = params_content.parse::<Type>()?;
                        
                        params.push(ResourceParam {
                            name: param_name,
                            param_type,
                        });
                        
                        if params_content.peek(Token![,]) {
                            params_content.parse::<Token![,]>()?;
                        }
                    }
                }
                
                let description = if content.lookahead1().peek(Ident) {
                    let peek_ident = content.fork().parse::<Ident>().ok();
                    if peek_ident.as_ref().map(|i| i.to_string()) == Some("with".to_string()) {
                        content.parse::<Ident>()?; // consume 'with'
                        Some(content.parse::<LitStr>()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                operations.push(ResourceOperation {
                    op_type,
                    name: op_name,
                    params,
                    description,
                });
                
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            
            resources.push(ResourceDefinition {
                name,
                operations,
            });
        }
        
        Ok(McpResourcesInput { resources })
    }
}

pub fn expand(input: McpResourcesInput) -> Result<TokenStream> {
    let McpResourcesInput { resources } = input;
    
    let resource_implementations = resources.iter().map(|resource| {
        let resource_name = &resource.name;
        let server_struct_name = format_ident!("{}McpServer", 
            resource_name.to_string().chars().next().unwrap().to_uppercase()
                .chain(resource_name.to_string().chars().skip(1)).collect::<String>());
        
        let operation_methods = resource.operations.iter().map(|op| {
            let method_name = &op.name;
            let params = op.params.iter().map(|p| {
                let name = &p.name;
                let ty = &p.param_type;
                quote! { #name: #ty }
            });
            
            let param_names = op.params.iter().map(|p| &p.name);
            let description = op.description.as_ref()
                .map(|d| d.value())
                .unwrap_or_else(|| format!("{} operation on {}", 
                    match op.op_type {
                        OperationType::List => "List",
                        OperationType::Read => "Read", 
                        OperationType::Write => "Write",
                    },
                    resource_name));
            
            match op.op_type {
                OperationType::List => {
                    quote! {
                        pub async fn #method_name(&self, #(#params),*) -> Result<Vec<ResourceItem>, McpError> {
                            let request = ListResourcesRequest {
                                resource_type: stringify!(#resource_name).to_string(),
                                #(#param_names),*
                            };
                            
                            self.handle_list_request(request).await
                        }
                    }
                }
                OperationType::Read => {
                    quote! {
                        pub async fn #method_name(&self, #(#params),*) -> Result<ResourceData, McpError> {
                            let request = ReadResourceRequest {
                                resource_type: stringify!(#resource_name).to_string(),
                                #(#param_names),*
                            };
                            
                            self.handle_read_request(request).await
                        }
                    }
                }
                OperationType::Write => {
                    quote! {
                        pub async fn #method_name(&self, #(#params),*) -> Result<WriteResult, McpError> {
                            let request = WriteResourceRequest {
                                resource_type: stringify!(#resource_name).to_string(),
                                #(#param_names),*
                            };
                            
                            self.handle_write_request(request).await
                        }
                    }
                }
            }
        });
        
        quote! {
            pub struct #server_struct_name {
                client: mcp_client::McpClient,
            }
            
            impl #server_struct_name {
                pub fn new(client: mcp_client::McpClient) -> Self {
                    Self { client }
                }
                
                #(#operation_methods)*
                
                async fn handle_list_request(&self, request: ListResourcesRequest) -> Result<Vec<ResourceItem>, McpError> {
                    self.client.list_resources(request).await
                        .map_err(McpError::ClientError)
                }
                
                async fn handle_read_request(&self, request: ReadResourceRequest) -> Result<ResourceData, McpError> {
                    self.client.read_resource(request).await
                        .map_err(McpError::ClientError)
                }
                
                async fn handle_write_request(&self, request: WriteResourceRequest) -> Result<WriteResult, McpError> {
                    self.client.write_resource(request).await
                        .map_err(McpError::ClientError)
                }
            }
        }
    });
    
    Ok(quote! {
        use serde::{Serialize, Deserialize};
        
        #(#resource_implementations)*
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ListResourcesRequest {
            pub resource_type: String,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ReadResourceRequest {
            pub resource_type: String,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct WriteResourceRequest {
            pub resource_type: String,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResourceItem {
            pub id: String,
            pub name: String,
            pub metadata: serde_json::Value,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResourceData {
            pub content: serde_json::Value,
            pub metadata: serde_json::Value,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct WriteResult {
            pub success: bool,
            pub message: String,
        }
        
        #[derive(Debug, thiserror::Error)]
        pub enum McpError {
            #[error("MCP client error: {0}")]
            ClientError(String),
            #[error("Resource not found: {0}")]
            ResourceNotFound(String),
            #[error("Permission denied: {0}")]
            PermissionDenied(String),
        }
    })
}

fn format_ident(name: &str) -> Ident {
    Ident::new(name, proc_macro2::Span::call_site())
}

#[allow(unused)]
fn format_ident_pascal_case(ident: &Ident) -> Ident {
    let s = ident.to_string();
    let pascal = s.chars().next().unwrap().to_uppercase().chain(s.chars().skip(1)).collect::<String>();
    Ident::new(&pascal, ident.span())
}