use lambubu::{CompoundEnvironment, Term, compile_term};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn term_to_tokens(term: &Term) -> TokenStream2 {
    match term {
        Term::Var(name) => quote! {Term::Var(#name.to_string())},
        Term::Abs(name, body) => {
            let b2 = term_to_tokens(body);
            quote! { Term::Abs(#name.to_string(), Box::new(#b2)) }
        }
        Term::Apply(a, b) => {
            let a2 = term_to_tokens(a);
            let b2 = term_to_tokens(b);
            quote! {Term::Apply(Box::new(#a2), Box::new(#b2))}
        }
    }
}

#[proc_macro]
pub fn term(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let s = lit.value();

    let compiled = match compile_term(&s, &CompoundEnvironment::new(vec![])) {
        Ok(term) => term,
        Err(_) => {
            return syn::Error::new(lit.span(), String::from("Invalid term"))
                .to_compile_error()
                .into();
        }
    };

    term_to_tokens(&compiled).into()
}
