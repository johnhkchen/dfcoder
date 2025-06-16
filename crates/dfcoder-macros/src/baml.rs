use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token, Result, Type, Field, FieldsNamed,
    braced, parenthesized,
};

pub struct BamlSchemaInput {
    pub schemas: Vec<SchemaDefinition>,
}

pub struct SchemaDefinition {
    pub name: Ident,
    pub categorize_as: Vec<CategoryDefinition>,
}

pub struct CategoryDefinition {
    pub name: Ident,
    pub subcategories: Vec<Ident>,
}

impl Parse for BamlSchemaInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut schemas = Vec::new();
        
        while !input.is_empty() {
            let name = input.parse::<Ident>()?;
            let categorize_ident: Ident = input.parse()?;
            if categorize_ident != "categorize" {
                return Err(syn::Error::new(categorize_ident.span(), "Expected 'categorize'"));
            }
            let as_ident: Ident = input.parse()?;
            if as_ident != "as" {
                return Err(syn::Error::new(as_ident.span(), "Expected 'as'"));
            }
            
            let content;
            braced!(content in input);
            
            let mut categorize_as = Vec::new();
            while !content.is_empty() {
                let category_name = content.parse::<Ident>()?;
                
                let subcategories_content;
                braced!(subcategories_content in content);
                
                let subcategories: Punctuated<Ident, Token![,]> = 
                    subcategories_content.parse_terminated(Ident::parse, Token![,])?;
                
                categorize_as.push(CategoryDefinition {
                    name: category_name,
                    subcategories: subcategories.into_iter().collect(),
                });
                
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            
            schemas.push(SchemaDefinition {
                name,
                categorize_as,
            });
            
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
        }
        
        Ok(BamlSchemaInput { schemas })
    }
}

pub fn expand(input: BamlSchemaInput) -> Result<TokenStream> {
    let BamlSchemaInput { schemas } = input;
    
    let schema_implementations = schemas.iter().map(|schema| {
        let schema_name = &schema.name;
        let category_enums = schema.categorize_as.iter().map(|category| {
            let category_name = &category.name;
            let subcategory_variants: Vec<_> = category.subcategories.iter().map(|sub| {
                let variant_name = format_ident_pascal_case(sub);
                quote! { #variant_name }
            }).collect();
            
            let subcategory_strings: Vec<_> = category.subcategories.iter().map(|sub| sub.to_string()).collect();
            let subcategory_match_arms: Vec<_> = category.subcategories.iter().zip(subcategory_strings.iter()).map(|(sub, s)| {
                let variant_name = format_ident_pascal_case(sub);
                quote! { #s => Some(Self::#variant_name) }
            }).collect();
            
            let subcategory_variants_2 = subcategory_variants.clone();
            
            quote! {
                #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                pub enum #category_name {
                    #(#subcategory_variants),*
                }
                
                impl #category_name {
                    pub fn from_str(s: &str) -> Option<Self> {
                        match s.to_lowercase().as_str() {
                            #(#subcategory_match_arms,)*
                            _ => None,
                        }
                    }
                    
                    pub fn as_str(&self) -> &'static str {
                        match self {
                            #(Self::#subcategory_variants_2 => stringify!(#subcategory_variants_2)),*
                        }
                    }
                }
            }
        });
        
        let classifier_struct_name = format_ident!("{}Classifier", schema_name);
        let category_types = schema.categorize_as.iter().map(|cat| &cat.name);
        
        quote! {
            #(#category_enums)*
            
            pub struct #classifier_struct_name {
                client: baml_client::BamlClient,
            }
            
            impl #classifier_struct_name {
                pub fn new(client: baml_client::BamlClient) -> Self {
                    Self { client }
                }
                
                pub async fn classify(&self, text: &str) -> Result<ClassificationResult, BamlError> {
                    let prompt = format!(
                        "Classify the following text into appropriate categories: {}",
                        text
                    );
                    
                    let response = self.client.invoke("classify_activity", &prompt).await?;
                    let classification = serde_json::from_str(&response)?;
                    Ok(classification)
                }
            }
            
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct ClassificationResult {
                #(pub #category_types: Option<#category_types>),*
            }
            
            #[derive(Debug, thiserror::Error)]
            pub enum BamlError {
                #[error("BAML client error: {0}")]
                ClientError(String),
                #[error("JSON parsing error: {0}")]
                JsonError(#[from] serde_json::Error),
            }
        }
    });
    
    Ok(quote! {
        use serde::{Serialize, Deserialize};
        
        #(#schema_implementations)*
    })
}

fn format_ident_pascal_case(ident: &Ident) -> Ident {
    let s = ident.to_string();
    let pascal = s.chars().next().unwrap().to_uppercase().chain(s.chars().skip(1)).collect::<String>();
    Ident::new(&pascal, ident.span())
}