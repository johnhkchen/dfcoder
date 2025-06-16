use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token, Result,
};

pub struct ScenarioInput {
    pub name: LitStr,
    pub given: Expr,
    pub when: Expr,
    pub then: Expr,
}

impl Parse for ScenarioInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<LitStr>()?;
        
        // Parse 'given' keyword
        let given_ident: Ident = input.parse()?;
        if given_ident != "given" {
            return Err(syn::Error::new(given_ident.span(), "Expected 'given'"));
        }
        input.parse::<Token![:]>()?;
        let given = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        
        // Parse 'when' keyword
        let when_ident: Ident = input.parse()?;
        if when_ident != "when" {
            return Err(syn::Error::new(when_ident.span(), "Expected 'when'"));
        }
        input.parse::<Token![:]>()?;
        let when = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        
        // Parse 'then' keyword
        let then_ident: Ident = input.parse()?;
        if then_ident != "then" {
            return Err(syn::Error::new(then_ident.span(), "Expected 'then'"));
        }
        input.parse::<Token![:]>()?;
        let then = input.parse::<Expr>()?;
        input.parse::<Token![;]>()?;
        
        Ok(ScenarioInput {
            name,
            given,
            when,
            then,
        })
    }
}

pub fn expand(input: ScenarioInput) -> Result<TokenStream> {
    let ScenarioInput { name, given, when, then } = input;
    let test_name = format!("scenario_{}", name.value().replace(" ", "_").to_lowercase());
    let test_ident = Ident::new(&test_name, name.span());
    
    Ok(quote! {
        #[cfg(test)]
        mod #test_ident {
            use super::*;
            use dfcoder_test_utils::*;
            
            #[tokio::test]
            async fn test() {
                let scenario = TestScenario::new(#name);
                
                // Setup conditions
                scenario.given(|| async {
                    #given
                }).await;
                
                // Trigger event
                scenario.when(|| async {
                    #when
                }).await;
                
                // Verify outcome
                scenario.then(|| async {
                    #then
                }).await;
                
                scenario.assert_success().await;
            }
        }
    })
}