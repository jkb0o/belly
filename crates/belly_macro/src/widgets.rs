use super::context::Context;
use super::ess::StyleSheet;
use super::ext::*;
use bevy::utils::HashMap;
use proc_macro2::{TokenStream, TokenTree};
use quote::*;
use syn::spanned::Spanned;

macro_rules! throw {
    ($span:expr, $msg:literal $($args:tt)*) => {
        return Err(syn::Error::new($span, format!($msg $($args)*)))
    };
}
macro_rules! ident {
    ($span:expr, $msg:literal $($args:tt)*) => {
        syn::Ident::new(format!($msg $($args)*).as_str(), $span)
    };
}

pub fn widget(ast: syn::ItemFn) -> Result<TokenStream, syn::Error> {
    let ctx = Context::new();
    let core = ctx.core_path();
    let attrs = WidgetAttributes::parse(&ast, &ctx)?;
    // fn button(...) expands into
    // button
    let widget_ident = &ast.sig.ident;
    // "button"
    let widget_name = widget_ident.to_string();
    // ButtonWidget
    let widget_struct = if widget_name.capitalized() {
        widget_ident.clone()
    } else {
        ident!(widget_ident.span(), "{}Widget", widget_name.to_camel_case())
    };
    // ButtonWidgetExtension
    let widget_extenstion = ident!(
        widget_ident.span(),
        "{}Extension",
        widget_struct.to_string()
    );

    let fn_args = &ast.sig.inputs;
    let fn_body = &ast.block;
    let mod_relations = format_ident!("{}_relations", widget_name.to_lowercase());
    // let allow_non_camel_case = if ctx.is_interal() {
    //     quote! { #[allow(non_camel_case_types)] }
    // } else {
    //     quote! {}
    // };

    // let type_components
    let components_associated_type = attrs.components.as_associated_type();
    let build_components_associated_type = attrs.build_components.as_associated_type();
    let rest_components_associated_type = attrs.rest_components.as_associated_type();
    let instantiate_components_impl = attrs.impl_instantiate_components()?;
    let split_components_impl = attrs.impl_split_components();

    let bindings_from_impl = attrs.impl_bindings_from();
    let bindings_from_deref = attrs.impl_bindings_from_deref();
    let bindings_to_impl = attrs.impl_bindings_to();
    let bindings_to_deref = attrs.impl_bindings_to_deref();
    let signals_impl = attrs.impl_signals();
    let signals_deref = attrs.impl_signals_deref();
    let default_styles_impl = attrs.impl_default_styles();
    let docs = attrs.build_docs();

    let alias = if let Some(extends) = &attrs.extends {
        quote!( Some(<#extends as #core::eml::Widget>::instance().name()) )
    } else {
        quote!(None)
    };
    let extends = if let Some(extends) = &attrs.extends {
        quote!(#extends)
    } else {
        quote!(#core::eml::build::DefaultWidget)
    };
    Ok(quote! {
        #docs
        pub struct #widget_struct;
        impl #core::eml::Widget for #widget_struct {
            type Components = #components_associated_type;
            type BuildComponents = #build_components_associated_type;
            type OtherComponents = #rest_components_associated_type;
            type BindingsFrom = #mod_relations::BindingsFrom;
            type BindingsTo = #mod_relations::BindingsTo;
            type Signals = #mod_relations::Signals;
            type Extends = #extends;

            fn instance() -> &'static Self {
                &#widget_struct
            }

            fn name(&self) -> #core::Tag {
                #core::tag!(#widget_name)
            }

            fn alias(&self) -> Option<#core::Tag> {
                #alias
            }

            fn build_widget(
                &self,
                ctx: &mut #core::eml::WidgetContext,
                components: &mut Self::BuildComponents
            ) {
                use #core::eml::BuildWidgetFunc;
                fn inner(#fn_args) {
                    #fn_body
                }
                inner.invoke_build_widget(ctx, components);
            }

            #instantiate_components_impl

            #split_components_impl

            #default_styles_impl
        }
        mod #mod_relations {
            pub struct BindingsFrom;
            impl #core::eml::Singleton for BindingsFrom {
                fn instance() -> &'static Self {
                    &BindingsFrom
                }
            }

            pub struct BindingsTo;
            impl #core::eml::Singleton for BindingsTo {
                fn instance() -> &'static Self {
                    &BindingsTo
                }
            }

            pub struct Signals;
            impl #core::eml::Singleton for Signals {
                fn instance() -> &'static Self {
                    &Signals
                }
            }
        }
        impl #mod_relations::BindingsFrom {
            #bindings_from_impl
        }
        impl ::std::ops::Deref for #mod_relations::BindingsFrom {
            #bindings_from_deref
        }
        impl #mod_relations::BindingsTo {
            #bindings_to_impl
        }
        impl ::std::ops::Deref for #mod_relations::BindingsTo {
            #bindings_to_deref
        }
        impl #mod_relations::Signals {
            #signals_impl
        }
        impl ::std::ops::Deref for #mod_relations::Signals {
            #signals_deref
        }
        pub trait #widget_extenstion {
            #docs
            #[allow(non_snake_case)]
            fn #widget_ident() -> &'static #widget_struct {
                &#widget_struct
            }
        }
        impl #widget_extenstion for #core::Widgets { }
    })
}

#[derive(Default)]
struct Components(Vec<syn::Type>);
impl Components {
    fn as_associated_type(&self) -> TokenStream {
        let mut body = quote! {};
        for component in self.0.iter() {
            body = quote! {
                #body #component,
            };
        }
        quote! { (#body) }
    }

    fn fetch_build_components(_ctx: &Context, ast: &syn::ItemFn) -> Result<Components, syn::Error> {
        let args = &ast.sig.inputs;
        let mut typs = vec![];
        for arg in args.iter().skip(1) {
            let span = arg.span();
            let syn::FnArg::Typed(arg) = arg else {
                throw!(span, "Expected `&mut Component`")
            };
            let syn::Type::Reference(arg) = arg.ty.as_ref().clone() else {
                throw!(span, "Expected `&mut Component`")
            };
            let typ = arg.elem.as_ref().clone();
            typs.push(typ);
        }
        Ok(Components(typs))
    }
}

struct Transfomer {
    ty: TokenStream,
    expr: TokenStream,
}

struct ParamTarget {
    component: syn::Type,
    property: Option<TokenStream>,
    transformer: Option<Transfomer>,
}

/// #[param(value: String => Label, value)]
struct Param {
    name: syn::Ident,
    ty: syn::Type,
    target: ParamTarget,
    docs: Vec<String>,
}

impl syn::parse::Parse for Param {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![:]>()?;
        let span = input.span();
        let ty = input
            .parse::<syn::Type>()
            .map_err(|e| syn::Error::new(span, format!("smth wrong: {e:?}")))?;
        input.parse::<syn::Token![=>]>()?;
        let component = input.parse::<syn::Type>()?;
        let property = if input.peek(syn::Token![:]) {
            input.parse::<syn::Token![:]>()?;
            Some(input.step(|cursor| {
                let mut rest = *cursor;
                let mut stream = quote! {};
                while let Some((tt, next)) = rest.token_tree() {
                    match &tt {
                        TokenTree::Punct(punct) if punct.as_char() == '|' => {
                            return Ok((stream, rest));
                        }
                        _ => rest = next,
                    };
                    stream = quote! { #stream #tt };
                }
                Ok((stream, rest))
            })?)
        } else {
            None
        };
        let transformer = if input.peek(syn::Token![|]) {
            input.parse::<syn::Token![|]>()?;
            let transformer = input.parse::<syn::Ident>()?;
            let ty = component.clone();
            if input.is_empty() {
                Some(Transfomer {
                    ty: quote! { #ty },
                    expr: quote! { #transformer },
                })
            } else {
                input.parse::<syn::Token![.]>()?;
                Some(Transfomer {
                    ty: quote! { #transformer },
                    expr: input.parse::<TokenStream>()?,
                })
            }
        } else {
            None
        };
        Ok(Param {
            name,
            ty,
            target: ParamTarget {
                component,
                property,
                transformer,
            },
            docs: vec![],
        })
        // let property = if lookahead.peek(Token![:]) {
        //     input.peek(token)
        // }
        //     name.clone()
        // } else {
        //     input.parse::<syn::Token![=>]>()?;
        //     input.parse::<syn::Ident>()?
        // };
        // Ok(Param::Proxy(name, prop_type, target))
    }
}

struct Signal {
    name: syn::Ident,
    ty: syn::Type,
    filter: TokenStream,
    docs: Vec<String>,
}

impl syn::parse::Parse for Signal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![:]>()?;
        let ty = input.parse::<syn::Type>()?;
        let filter = if input.is_empty() {
            quote! { |_| true }
        } else {
            input.parse::<syn::Token![=>]>()?;
            input.parse::<TokenStream>()?
        };
        Ok(Signal {
            name,
            ty,
            filter,
            docs: vec![],
        })
    }
}

// impl<'a> Param<'a> {
//     fn parse(context: &'a Context, attr: &syn::Attribute) -> Result<Param<'a>, syn::Error> {
//         let args = attr.parse_args::<TokenStream>()?;

//         throw!(attr.span(), "Smth wrong")
//     }
// }

struct AttributeValue<T> {
    pub value: T,
}

impl<T: syn::parse::Parse> syn::parse::Parse for AttributeValue<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![=]>()?;
        Ok(AttributeValue {
            value: input.parse()?,
        })
    }
}

enum DefaultStyles {
    Reference(syn::Ident),
    Value(StyleSheet),
}

impl DefaultStyles {
    fn new() -> DefaultStyles {
        DefaultStyles::Value(StyleSheet::default())
    }

    fn as_tokens(&self) -> TokenStream {
        match self {
            Self::Reference(ident) => quote!(#ident),
            Self::Value(sheet) => {
                let value = format!("{sheet:#}");
                quote!(#value)
            }
        }
    }
}

struct WidgetAttributes<'a> {
    ctx: &'a Context,
    name: String,
    components: Components,
    build_components: Components,
    rest_components: Components,
    params: Vec<Param>,
    signals: HashMap<String, Signal>,
    default_styles: DefaultStyles,
    extends: Option<syn::Type>,
    docs: Vec<String>,
}

impl<'a> WidgetAttributes<'a> {
    fn parse(ast: &syn::ItemFn, context: &'a Context) -> Result<WidgetAttributes<'a>, syn::Error> {
        let mut attrs = WidgetAttributes {
            ctx: context,
            name: ast.sig.ident.to_string(),
            components: Components::default(),
            build_components: Components::fetch_build_components(context, ast)?,
            rest_components: Components::default(),
            params: Vec::new(),
            signals: HashMap::new(),
            default_styles: DefaultStyles::new(),
            extends: None,
            docs: vec![],
        };
        let mut docs = vec![];
        for attr in ast.attrs.iter() {
            if attr.path.is_ident("doc") {
                let doc: AttributeValue<syn::LitStr> = syn::parse2(attr.tokens.clone())?;
                docs.push(doc.value.value())
            } else if attr.path.is_ident("param") {
                let mut param = attr.parse_args::<Param>()?;
                if !attrs.build_components.0.contains(&param.target.component) {
                    if !attrs.rest_components.0.contains(&param.target.component) {
                        attrs.rest_components.0.push(param.target.component.clone());
                    }
                }
                let name = param.name.to_string();
                if attrs
                    .params
                    .iter()
                    .filter(|p| p.name == name)
                    .next()
                    .is_some()
                {
                    throw!(attr.span(), "Param `{name}` already defined")
                }
                param.docs = docs;
                docs = vec![];
                attrs.params.push(param);
            } else if attr.path.is_ident("signal") {
                let mut signal = attr.parse_args::<Signal>()?;
                signal.docs = docs;
                docs = vec![];
                attrs.signals.insert(signal.name.to_string(), signal);
            } else if attr.path.is_ident("styles") {
                if let Ok(AttributeValue::<syn::Ident> { value }) = syn::parse2(attr.tokens.clone())
                {
                    attrs.default_styles = DefaultStyles::Reference(value)
                } else {
                    attrs.default_styles = DefaultStyles::Value(attr.parse_args()?)
                }
            } else if attr.path.is_ident("extends") {
                attrs.extends = Some(attr.parse_args()?);
            }
        }

        attrs.docs = docs;
        attrs.components.0.extend(attrs.build_components.0.clone());
        attrs.components.0.extend(attrs.rest_components.0.clone());
        Ok(attrs)
    }

    fn impl_default_styles(&self) -> TokenStream {
        let styles = self.default_styles.as_tokens();
        quote! {
            fn default_styles(&self) -> &str {
                #styles
            }
        }
    }

    fn impl_split_components(&self) -> TokenStream {
        let mut all_components = quote! {};
        let mut build_components = quote! {};
        let mut rest_components = quote! {};
        for idx in 0..self.components.0.len() {
            let cname = format_ident!("c{idx}");
            all_components = quote! { #all_components #cname, };
        }
        for idx in 0..self.build_components.0.len() {
            let cname = format_ident!("c{idx}");
            build_components = quote! { #build_components #cname, };
        }
        for idx in 0..self.rest_components.0.len() {
            let cname = format_ident!("c{idx}");
            rest_components = quote! { #rest_components #cname, };
        }
        quote! {
            fn split_components(&self, components: Self::Components) -> (Self::BuildComponents, Self::OtherComponents) {
                let (#all_components) = components;
                ((#build_components), (#rest_components))
            }

        }
    }

    fn impl_bindings_from(&self) -> TokenStream {
        let core = self.ctx.core_path();
        let mut body = quote! {};
        for param in self.params.iter() {
            let ident = &param.name;
            let ty = &param.ty;
            let component = &param.target.component;
            let mut bind = quote! { #component };
            let mut sep = quote! {};
            if let Some(prop) = &param.target.property {
                bind = quote! { #bind:#prop };
                sep = quote!( . );
            } else {
                bind = quote! { #bind: };
            }
            if let Some(transform) = &param.target.transformer {
                let method = &transform.expr;
                bind = quote! { #bind #sep #method() }
            }
            body = quote! {
                #body
                pub fn #ident(&self, entity: Entity) -> #core::relations::bind::FromComponent<#component, #ty> {
                    #core::from!(entity, #bind)
                }
            }
        }
        quote! {
            #body
        }
    }

    fn impl_bindings_from_deref(&self) -> TokenStream {
        let core = self.ctx.core_path();
        if let Some(ty) = &self.extends {
            quote! {
                type Target = <#ty as #core::eml::Widget>::BindingsFrom;
                fn deref(&self) -> &Self::Target {
                    #ty::instance().bind_from()
                }
            }
        } else {
            quote! {
                type Target = #core::eml::DefaultBindingsFrom;
                fn deref(&self) -> &Self::Target {
                    &#core::eml::DefaultBindingsFrom
                }
            }
        }
    }

    fn impl_bindings_to(&self) -> TokenStream {
        let core = self.ctx.core_path();
        let mut body = quote! {};
        for param in self.params.iter() {
            let ident = &param.name;
            let ty = &param.ty;
            let component = &param.target.component;
            let mut bind_type = quote! { ToComponentWithoutTransformer };
            let mut bind_args = quote! { #ty };
            let mut bind = quote! { #component };
            if let Some(prop) = &param.target.property {
                bind = quote! { #bind:#prop }
            }
            if let Some(transform) = &param.target.transformer {
                let bind_ty = &transform.ty;
                let method = &transform.expr;
                bind_type = quote! { ToComponent };
                bind_args = quote! { #ty, #bind_ty };
                bind = quote! { #bind|#method }
            }
            body = quote! {
                #body
                pub fn #ident(&self, entity: Entity) -> #core::relations::bind::#bind_type<#component, #bind_args> {
                    #core::to!(entity, #bind)
                }
            }
        }
        body
    }

    fn impl_bindings_to_deref(&self) -> TokenStream {
        let core = self.ctx.core_path();
        if let Some(ty) = &self.extends {
            quote! {
                type Target = <#ty as #core::eml::Widget>::BindingsTo;
                fn deref(&self) -> &Self::Target {
                    #ty::instance().bind_to()
                }
            }
        } else {
            quote! {
                type Target = #core::eml::DefaultBindingsTo;
                fn deref(&self) -> &Self::Target {
                    &#core::eml::DefaultBindingsTo
                }
            }
        }
    }

    fn impl_signals(&self) -> TokenStream {
        let core = self.ctx.core_path();
        let mut body = quote! {};
        for signal in self.signals.values() {
            let name = &signal.name;
            let event = &signal.ty;
            let filter = &signal.filter;
            body = quote! {
                #body
                pub fn #name(&self) -> #core::relations::connect::EventFilter<#event> {
                    #core::relations::connect::EventFilter::Entity(#filter)
                }
            };
        }
        body
    }

    fn impl_signals_deref(&self) -> TokenStream {
        let core = self.ctx.core_path();
        if let Some(ty) = &self.extends {
            quote! {
                type Target = <#ty as #core::eml::Widget>::Signals;
                fn deref(&self) -> &Self::Target {
                    #ty::instance().on()
                }
            }
        } else {
            quote! {
                type Target = #core::eml::DefaultSignals;
                fn deref(&self) -> &Self::Target {
                    &#core::eml::DefaultSignals
                }
            }
        }
    }

    fn impl_instantiate_components(&self) -> Result<TokenStream, syn::Error> {
        let core = self.ctx.core_path();
        let widget_name = &self.name;
        let mut instantiate_body = quote! {};
        for component in self.components.0.iter() {
            let mut params = quote! {
                let mut component_params = #core::eml::Params::default();
            };
            let mut setters = quote! {};
            for param in self
                .params
                .iter()
                .filter(|p| component == &p.target.component)
            {
                let param_name = param.name.to_string();
                let param_type = &param.ty;
                params = quote! {
                    #params
                    if let Some(component_param) = params.drop_variant(#param_name.into()) {
                        component_params.insert(#param_name, component_param);
                    }
                };
                let mut prop_body = quote! { component };
                if let Some(getter) = &param.target.property {
                    prop_body = quote! { #prop_body.#getter }
                }
                if let Some(transformer) = &param.target.transformer {
                    let tr_type = &transformer.ty;
                    let tr_path = &transformer.expr;
                    prop_body = quote! {
                        {
                            let transform = #tr_type::get_properties().#tr_path().as_transformer();
                            if let Err(e) = transform(&value, (&mut #prop_body).into()) {
                                ::bevy::prelude::error!("Can't transform property {}: {}", #param_name, e)
                            }
                        }
                    }
                } else {
                    prop_body = quote! { #prop_body = value.into() };
                }
                setters = quote! {
                    #setters
                    if let Some(value) = component_params.drop_variant(#param_name.into()) {
                        match #param_type::try_from(value) {
                            Ok(value) => #prop_body,
                            Err(err) => ::bevy::prelude::error!("Can't set {}.{}: {}", #widget_name, #param_name, err),
                        }
                    }
                }
            }
            instantiate_body = quote! {
                #instantiate_body
                {
                    #params
                    let mut component = #component::from_world_and_params(world, &mut component_params);
                    #setters
                    component
                },
            }
        }

        Ok(quote! {
            fn instantiate_components(
                &self,
                world: &mut ::bevy::prelude::World,
                params: &mut #core::eml::Params
            ) -> Self::Components {
                (#instantiate_body)
            }
        })
    }

    pub fn build_docs(&self) -> TokenStream {
        let name = format!(" <!-- @widget-name={} -->", self.name.to_string());
        let mut docs = quote! {
            #[doc = #name]
            #[doc = " <!-- @widget-body-begin -->"]
        };
        for doc in self.docs.iter() {
            docs = quote! {
                #docs
                #[doc = #doc]
            }
        }
        docs = quote! {
            #docs
            #[doc = " <!-- @widget-body-end -->"]
        };
        if !self.params.is_empty() {
            docs = quote! {
                #docs
                #[doc = " "]
                #[doc = " Params:"]
            }
        }
        docs = quote! {
            #docs
            #[doc = " <!-- @widget-params-begin -->"]
        };
        for param in self.params.iter() {
            let param_signature = format!(
                " - `{}:` [`{}`]",
                param.name.to_string(),
                param.ty.to_token_stream().to_string().replace(" ", "")
            );
            docs = quote! {
                #docs
                #[doc = #param_signature]
            };
            for doc in param.docs.iter() {
                docs = quote! {
                    #docs
                    #[doc = #doc]
                }
            }
            docs = quote! {
                #docs
                #[doc = " "]
            }
        }
        docs = quote! {
            #docs
            #[doc = " <!-- @widget-params-end -->"]
        };

        if !self.signals.is_empty() {
            docs = quote! {
                #docs
                #[doc = " "]
                #[doc = " Signals:"]
            }
        }
        docs = quote! {
            #docs
            #[doc = " <!-- @widget-signals-begin -->"]
        };
        for signal in self.signals.values() {
            let signal_signature = format!(
                " - `{}:` [`{}`]",
                signal.name.to_string(),
                signal.ty.to_token_stream().to_string().replace(" ", "")
            );
            docs = quote! {
                #docs
                #[doc = #signal_signature]
            };
            for doc in signal.docs.iter() {
                docs = quote! {
                    #docs
                    #[doc = #doc]
                }
            }
            docs = quote! {
                #docs
                #[doc = " "]
            }
        }

        docs = quote! {
            #docs
            #[doc = " <!-- @widget-signals-end -->"]
        };
        docs
    }
}
