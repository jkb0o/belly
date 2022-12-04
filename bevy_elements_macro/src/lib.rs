use proc_macro2::{Span, TokenStream};
use quote::*;
extern crate proc_macro;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error, Expr, ExprPath, ItemFn};
use syn_rsx::{parse, Node, NodeAttribute};

fn create_single_command_stmt(expr: &ExprPath) -> TokenStream {
    let component_span = expr.span();
    if let Some(component) = expr.path.get_ident() {
        if component.to_string().chars().next().unwrap().is_uppercase() {
            quote_spanned! {component_span=>
                c.insert(#component::default());
            }
        } else {
            quote_spanned! {component_span=>
                c.insert(#component);
            }
        }
    } else {
        Error::new(component_span, "Invalid components declaration").into_compile_error()
    }
}

fn create_command_stmts(expr: &Expr) -> TokenStream {
    let with_body = match expr {
        Expr::Path(path) => create_single_command_stmt(path),
        Expr::Tuple(components) => {
            let mut components_expr = quote! {};
            for component_expr in components.elems.iter() {
                let component_span = component_expr.span();
                if let Expr::Path(component) = component_expr {
                    let component_expr = create_single_command_stmt(component);
                    components_expr = quote_spanned! {component_span=>
                        #components_expr
                        #component_expr
                    };
                } else {
                    return Error::new(component_span, "Invalid component name")
                        .into_compile_error();
                }
            }
            components_expr
        }
        _ => {
            return Error::new(expr.span(), "Invalid components declaration").into_compile_error();
        }
    };
    let expr_span = expr.span();
    quote_spanned! {expr_span=>
        __ctx.attributes.add(::bevy_elements_core::attributes::Attribute::from_commands("with", ::std::boxed::Box::new(move |c| {
            #with_body
        })));
    }
}

fn create_attr_stmt(attr: &NodeAttribute) -> TokenStream {
    let attr_name = attr.key.to_string();
    match &attr.value {
        None => {
            return quote! {
                __ctx.attributes.add(::bevy_elements_core::attributes::Attribute::new(
                    #attr_name.into(),
                    ::bevy_elements_core::attributes::AttributeValue::Empty
                ));
            };
        }
        Some(attr_value) => {
            let attr_value = attr_value.as_ref();
            let attr_span = attr_value.span();
            if attr_name == "with" {
                return create_command_stmts(attr_value);
            } else {
                return quote_spanned! {attr_span=>
                    __ctx.attributes.add(::bevy_elements_core::attributes::Attribute::new(
                        #attr_name.into(),
                        (#attr_value).into()
                    ));
                };
            }
        }
    }
}

fn walk_nodes<'a>(element: &'a Node, create_entity: bool) -> TokenStream {
    let mut children = quote! {};
    let mut parent = if create_entity {
        quote! { let __parent = __world.spawn_empty().id(); }
    } else {
        quote! {}
    };
    if let Node::Element(element) = element {
        let mut parent_defined = false;
        for attr in element.attributes.iter() {
            if let Node::Block(entity) = attr {
                let entity_span = entity.value.span();
                let entity = entity.value.as_ref();
                if parent_defined {
                    return Error::new(entity_span, "Entity already provided by entity attribute")
                        .into_compile_error();
                }
                parent_defined = true;
                parent = quote! {
                    let __parent = #entity;
                };
            } else if let Node::Attribute(attr) = attr {
                let attr_name = attr.key.to_string();
                if &attr_name == "entity" {
                    let attr_span = attr.key.span();
                    if parent_defined {
                        return Error::new(attr_span, "Entity already provided by braced block")
                            .into_compile_error();
                    }
                    parent_defined = true;
                    let attr_value = attr.value.as_ref();
                    if attr_value.is_none() {
                        return Error::new(attr_span, "Attriute entity should has a value")
                            .into_compile_error();
                    }
                    let entity = attr_value.unwrap().as_ref();
                    parent = quote_spanned! { attr_span=>
                        let __parent = #entity;
                    };
                } else {
                    let attr_stmt = create_attr_stmt(attr);
                    children = quote! {
                        #children
                        #attr_stmt
                    };
                }
            }
        }
        for child in element.children.iter() {
            match child {
                Node::Element(_) => {
                    let expr = walk_nodes(child, true);
                    children = quote! {
                        #children
                        __ctx.children.push( #expr );
                    };
                }
                Node::Text(text) => {
                    let text = text.value.as_ref();
                    children = quote! {
                        #children
                        __ctx.children.push(
                            __world.spawn(::bevy::prelude::TextBundle {
                                text: ::bevy::prelude::Text::from_section(
                                    #text,
                                    ::std::default::Default::default()
                                ),
                                ..default()
                            })
                            .insert(::bevy_elements_core::Element::inline())
                            .id()
                        );
                    };
                }
                Node::Block(block) => {
                    let block = block.value.as_ref();
                    let block_span = block.span();
                    children = quote_spanned! { block_span=>
                        #children
                        let __node = __world.spawn_empty().id();
                        for __child in #block.into_content(__node, __world).iter() {
                            __ctx.children.push( __child.clone() );
                        }
                    }
                }
                _ => (),
            };
        }

        let tag = syn::Ident::new(&element.name.to_string(), element.span());
        quote! {
            {
                #parent
                let mut __ctx = ::bevy_elements_core::eml::build::ElementContextData::new(__parent);

                #children
                let __builder = ::bevy_elements_core::Elements::#tag();
                __builder.build(__world, __ctx);
                __parent
            }
        }
    } else {
        quote! {}
    }
}

#[proc_macro]
pub fn eml(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(tree.into()) {
        Err(err) => err.to_compile_error().into(),
        Ok(nodes) => {
            let body = walk_nodes(&nodes[0], false);
            // nodes[0]
            let wraped = quote! {
                ::bevy_elements_core::ElementsBuilder::new(
                    move |
                        __world: &mut ::bevy::prelude::World,
                        __parent: ::bevy::prelude::Entity
                    | {
                        #body;
                    }
            )};
            proc_macro::TokenStream::from(wraped)
        }
    }
}

fn err(span: Span, message: &str) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(syn::Error::new(span, message).to_compile_error())
}

#[proc_macro_derive(Widget, attributes(alias, param))]
pub fn widget_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let span = ast.span();

    let component = ast.ident;
    let component_str = format!("{component}");
    let mut alias_expr = quote! { #component_str };
    let extension_ident = format_ident!("{component}WidgetExtension");

    let mut docs = quote! {};
    let mut construct_body = quote! {};
    // TODO: use `doclines` to generate XSD docs like extension docs
    let mut doclines: Vec<String> = vec![];
    let mut bind_body = quote! {
        let this = ctx.entity();
    };
    let mut extension_body = quote! {
        #[doc = " This is realy cool thing"]
        #[allow(non_snake_case)]
        fn #component() -> ::bevy_elements_core::ElementBuilder {
            #component::as_builder()
        }
    };

    let syn::Data::Struct(data) = ast.data else {
        return err(span, "Widget could be derived only for structs");
    };

    for field in data.fields.iter() {
        let Some(field_ident) = field.ident.as_ref() else {
            return err(span, "Tuple Structs not yet supported by Widget derive.")
        };
        let field_str = format!("{field_ident}");

        if let syn::Type::Path(path) = &field.ty {
            let type_repr = path.clone().into_token_stream().to_string();
            if &type_repr == "Entity" {
                if &field_str == "ctx" {
                    return err(
                        field.span(),
                        "Using `ctx` as field name may lead to Widget's unexpected behaviour.",
                    );
                }
                if &field_str == "this" {
                    return err(
                        field.span(),
                        "Using `this` as field name may lead to Widget's unexpected behaviour.",
                    );
                }
                construct_body = quote! {
                    #construct_body
                    #field_ident: world.spawn_empty().id(),
                };
                bind_body = quote! {
                    #bind_body
                    let #field_ident = self.#field_ident;
                };
                continue;
            }
        }
        construct_body = quote! {
            #construct_body
            #field_ident: ::std::default::Default::default(),
        }
    }

    for field in data.fields.iter() {
        let field_ident = &field.ident;
        for attr in field.attrs.iter() {
            if !attr.path.is_ident("param") {
                continue;
            };
            let field_type = &field.ty;
            bind_body = quote! {
                #bind_body
                if let ::std::option::Option::Some(value) =
                    ::bevy_elements_core::bindattr!(ctx, #field_ident:#field_type => Self.#field_ident)
                {
                    self.#field_ident = value;
                }
            };
            if !attr.tokens.is_empty() {
                let bind_target = attr.parse_args::<TokenStream>().unwrap();
                bind_body = quote! {
                    #bind_body
                    ctx.commands().add(
                        ::bevy_elements_core::bind!(this, #component.#field_ident #bind_target)
                    );
                }
            }
        }
    }

    for attr in ast.attrs.iter().filter(|a| a.path.is_ident("doc")) {
        let docline = attr.tokens.to_string();
        if docline.starts_with("= ") {
            doclines.push(docline[2..].to_string());
        }
        docs = quote! {
            #docs
            #attr
        }
    }

    for attr in ast.attrs.iter() {
        if attr.path.is_ident("alias") {
            let Ok(alias) = attr.parse_args::<syn::Ident>() else {
                return err(attr.span(), "Alias should be defined using tokens: `#[alias(alias_name)]");
            };
            let alias_str = format!("{alias}");
            alias_expr = quote! { #alias_expr, #alias_str };
            extension_body = quote! {
                #extension_body
                #docs
                fn #alias() -> ::bevy_elements_core::ElementBuilder {
                    #component::as_builder()
                }
            }
        }
    }

    proc_macro::TokenStream::from(quote! {
        impl ::bevy_elements_core::Widget for #component {
            fn names() -> &'static [&'static str] {
                &[#alias_expr]
            }
            fn construct_component(world: &mut ::bevy::prelude::World) -> ::std::option::Option<Self> {
                ::std::option::Option::Some(#component {
                    #construct_body
                })
            }
            fn bind_component(&mut self, ctx: &mut ::bevy_elements_core::ElementContext) {
                #bind_body
            }
        }

        pub trait #extension_ident {
            #extension_body
        }

        impl #extension_ident for ::bevy_elements_core::Elements { }
    })
}
