use proc_macro2::TokenStream;
use quote::*;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Token;

macro_rules! throw {
    ($span:expr, $msg:literal $($args:tt)*) => {
        return Err(syn::Error::new($span, format!($msg $($args)*)))
    };
}

pub struct Run {
    target: Option<syn::Ident>,
    ctx: Option<syn::Pat>,
    system_args: Vec<syn::FnArg>,
    body: TokenStream,
}

impl Run {
    pub fn build(&self) -> TokenStream {
        let target = if let Some(target) = &self.target {
            quote! { Some(#target) }
        } else {
            quote! { None }
        };
        let ctx = if let Some(ctx) = &self.ctx {
            quote! { #ctx }
        } else {
            quote! { _ }
        };
        let mut types = quote! {};
        let mut sys_args = quote! {};
        for arg in self.system_args.iter() {
            let syn::FnArg::Typed(arg) = arg else {
                continue;
            };
            let arg_pat = &arg.pat;
            let arg_type = &arg.ty;
            sys_args = quote! { #sys_args #arg_pat, };
            types = quote! { #types #arg_type, };
        }
        let body = &self.body;
        quote! {
            (::std::marker::PhantomData::<(#types)>, #target, move |#ctx, (#sys_args)| {
                #body;
            })
        }
    }
}

fn args_done(i: &mut syn::parse::ParseStream) -> bool {
    if i.peek(Token![|]) {
        i.parse::<Token![|]>().unwrap();
        true
    } else {
        false
    }
}
impl syn::parse::Parse for Run {
    fn parse(mut input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target = if let Ok(ident) = input.call(syn::Ident::parse_any) {
            if ident.to_string() != "for" {
                throw!(ident.span(), "Expected closure or 'for' keyword");
            }
            Some(input.parse()?)
        } else {
            None
        };
        input.parse::<Token![|]>()?;
        let mut system_args = vec![];
        let mut ctx = None;
        let mut body = quote! {};
        while let Ok(pat) = input.parse() {
            if input.peek(Token![;]) {
                body = quote! { #pat };
                break;
            }
            if input.peek(Token![:]) {
                system_args.push(syn::FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: Box::new(pat),
                    colon_token: input.parse()?,
                    ty: Box::new(input.parse()?),
                }));
            } else if !system_args.is_empty() {
                throw!(pat.span(), "Invalid system args sequesnce for asyn! func")
            } else if ctx.is_none() {
                ctx = Some(pat);
            } else {
                throw!(pat.span(), "Invalid system arg sequesnce for asyn! func")
            }
            if input.peek(Comma) {
                input.parse::<Comma>()?;
            }
            if args_done(&mut input) {
                break;
            }
        }

        if ctx.is_some() || !system_args.is_empty() {
            args_done(&mut input);
        }
        let rest_body = input.parse::<TokenStream>()?;
        body = quote! { #body #rest_body };
        Ok(Run {
            target,
            ctx,
            system_args,
            body,
        })
    }
}
