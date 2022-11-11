extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn tag(name: TokenStream) -> TokenStream {
    let name = parse_macro_input!(name as LitStr);
    TokenStream::from(quote! {
        unsafe {
            static mut __tag: ::tagstr_core::Tag = ::tagstr_core::undefined_tag();
            static __once: ::std::sync::Once = ::std::sync::Once::new();
            __once.call_once(|| {
                __tag = ::tagstr_core::tag( #name );
            });
            __tag
        }
    })
}