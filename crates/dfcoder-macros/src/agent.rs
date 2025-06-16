use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token, Result, Path,
};

pub struct AgentInput {
    pub name: Ident,
    pub behaviors: Vec<AgentBehavior>,
}

pub struct AgentBehavior {
    pub trigger: BehaviorTrigger,
    pub action: Expr,
}

pub enum BehaviorTrigger {
    RespondsTo(LitStr),
    WhenIdle,
    DuringSupervision,
}

impl Parse for AgentInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let mut behaviors = Vec::new();
        
        while !input.is_empty() {
            let behavior = parse_behavior(input)?;
            behaviors.push(behavior);
            
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(AgentInput { name, behaviors })
    }
}

fn parse_behavior(input: ParseStream) -> Result<AgentBehavior> {
    let lookahead = input.lookahead1();
    let trigger = if lookahead.peek(Ident) {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "responds" => {
                // Parse 'responds to "pattern"'
                let to_ident: Ident = input.parse()?;
                if to_ident != "to" {
                    return Err(syn::Error::new(to_ident.span(), "Expected 'to' after 'responds'"));
                }
                let pattern = input.parse::<LitStr>()?;
                BehaviorTrigger::RespondsTo(pattern)
            }
            "when" => {
                // Parse 'when idle'
                let keyword = input.parse::<Ident>()?;
                match keyword.to_string().as_str() {
                    "idle" => BehaviorTrigger::WhenIdle,
                    _ => return Err(syn::Error::new(keyword.span(), "Expected 'idle' after 'when'")),
                }
            }
            "during" => {
                // Parse 'during supervision'
                let keyword = input.parse::<Ident>()?;
                match keyword.to_string().as_str() {
                    "supervision" => BehaviorTrigger::DuringSupervision,
                    _ => return Err(syn::Error::new(keyword.span(), "Expected 'supervision' after 'during'")),
                }
            }
            _ => return Err(syn::Error::new(ident.span(), "Expected 'responds', 'when', or 'during'"))
        }
    } else {
        return Err(lookahead.error());
    };
    
    input.parse::<Token![:]>()?;
    let action = input.parse::<Expr>()?;
    
    Ok(AgentBehavior { trigger, action })
}

pub fn expand(input: AgentInput) -> Result<TokenStream> {
    let AgentInput { name, behaviors } = input;
    let struct_name = name.clone();
    let mod_name = Ident::new(&format!("{}_impl", name.to_string().to_lowercase()), name.span());
    
    let behavior_methods = behaviors.iter().map(|behavior| {
        match &behavior.trigger {
            BehaviorTrigger::RespondsTo(pattern) => {
                let action = &behavior.action;
                quote! {
                    async fn handle_response_trigger(&self, input: &str) -> Option<AgentResponse> {
                        if input.contains(#pattern) {
                            Some(#action.await)
                        } else {
                            None
                        }
                    }
                }
            }
            BehaviorTrigger::WhenIdle => {
                let action = &behavior.action;
                quote! {
                    async fn handle_idle(&self) -> AgentAction {
                        #action.await
                    }
                }
            }
            BehaviorTrigger::DuringSupervision => {
                let action = &behavior.action;
                quote! {
                    async fn handle_supervision(&self, context: &SupervisionContext) -> SupervisionResponse {
                        #action.await
                    }
                }
            }
        }
    });
    
    Ok(quote! {
        pub struct #struct_name {
            id: AgentId,
            name: String,
            state: AgentState,
        }
        
        impl #struct_name {
            pub fn new(id: AgentId, name: impl Into<String>) -> Self {
                Self {
                    id,
                    name: name.into(),
                    state: AgentState::default(),
                }
            }
            
            #(#behavior_methods)*
        }
        
        impl Agent for #struct_name {
            fn id(&self) -> AgentId {
                self.id
            }
            
            fn name(&self) -> &str {
                &self.name
            }
            
            fn state(&self) -> &AgentState {
                &self.state
            }
        }
    })
}