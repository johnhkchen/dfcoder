use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token, Result, Type, Field, FieldsNamed,
};

pub struct EventsInput {
    pub events: Vec<EventDefinition>,
}

pub struct EventDefinition {
    pub from: Ident,
    pub to: Ident,
    pub event_name: Ident,
    pub fields: Option<FieldsNamed>,
}

impl Parse for EventsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut events = Vec::new();
        
        while !input.is_empty() {
            // Parse 'from Agent to Supervisor: EventName { fields }'
            let from_ident: Ident = input.parse()?;
            if from_ident != "from" {
                return Err(syn::Error::new(from_ident.span(), "Expected 'from'"));
            }
            
            let from = input.parse::<Ident>()?;
            
            let to_ident: Ident = input.parse()?;
            if to_ident != "to" {
                return Err(syn::Error::new(to_ident.span(), "Expected 'to'"));
            }
            
            let to = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            let event_name = input.parse::<Ident>()?;
            
            let fields = if input.peek(syn::token::Brace) {
                Some(input.parse::<FieldsNamed>()?)
            } else {
                None
            };
            
            events.push(EventDefinition {
                from,
                to,
                event_name,
                fields,
            });
            
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(EventsInput { events })
    }
}

pub fn expand(input: EventsInput) -> Result<TokenStream> {
    let EventsInput { events } = input;
    
    let event_enums = events.iter().map(|event| {
        let event_name = &event.event_name;
        let from = &event.from;
        let to = &event.to;
        
        match &event.fields {
            Some(fields) => {
                quote! {
                    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                    pub struct #event_name #fields
                    
                    impl Event for #event_name {
                        type Source = #from;
                        type Target = #to;
                        
                        fn source_type() -> &'static str {
                            stringify!(#from)
                        }
                        
                        fn target_type() -> &'static str {
                            stringify!(#to)
                        }
                        
                        fn event_type() -> &'static str {
                            stringify!(#event_name)
                        }
                    }
                }
            }
            None => {
                quote! {
                    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                    pub struct #event_name;
                    
                    impl Event for #event_name {
                        type Source = #from;
                        type Target = #to;
                        
                        fn source_type() -> &'static str {
                            stringify!(#from)
                        }
                        
                        fn target_type() -> &'static str {
                            stringify!(#to)
                        }
                        
                        fn event_type() -> &'static str {
                            stringify!(#event_name)
                        }
                    }
                }
            }
        }
    });
    
    let event_enum_variants = events.iter().map(|event| {
        let event_name = &event.event_name;
        match &event.fields {
            Some(_) => quote! { #event_name(#event_name) },
            None => quote! { #event_name },
        }
    });
    
    Ok(quote! {
        use serde::{Serialize, Deserialize};
        
        #(#event_enums)*
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum SystemEvent {
            #(#event_enum_variants),*
        }
        
        pub trait Event: Send + Sync + Clone + std::fmt::Debug {
            type Source;
            type Target;
            
            fn source_type() -> &'static str;
            fn target_type() -> &'static str;
            fn event_type() -> &'static str;
        }
        
        pub trait EventHandler<E: Event> {
            async fn handle(&self, event: E) -> Result<(), EventError>;
        }
        
        #[derive(Debug, thiserror::Error)]
        pub enum EventError {
            #[error("Handler not found for event type: {0}")]
            HandlerNotFound(String),
            #[error("Event processing failed: {0}")]
            ProcessingFailed(String),
        }
    })
}