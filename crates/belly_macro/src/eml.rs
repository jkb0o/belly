use proc_macro2::TokenStream;
use quote::*;
use syn::{spanned::Spanned, Expr, ExprPath, Ident};
use syn_rsx::{Node, NodeAttribute, NodeElement};

use super::context::*;

macro_rules! throw {
    ($span:expr, $msg:literal $($args:tt)*) => {
        return Err(syn::Error::new($span, format!($msg $($args)*)))
    };
}

fn create_single_command_stmt(expr: &ExprPath) -> syn::Result<TokenStream> {
    let component_span = expr.span();
    if let Some(component) = expr.path.get_ident() {
        if component.to_string().chars().next().unwrap().is_uppercase() {
            Ok(quote_spanned! {component_span=>
                c.insert(#component::default());
            })
        } else {
            Ok(quote_spanned! {component_span=>
                c.insert(#component);
            })
        }
    } else {
        throw!(component_span, "Invalid components declaration")
    }
}

fn create_command_stmts(ctx: &Context, expr: &Expr) -> syn::Result<TokenStream> {
    let core = ctx.core_path();
    let with_body = match expr {
        Expr::Path(path) => create_single_command_stmt(path)?,
        Expr::Tuple(components) => {
            let mut components_expr = quote! {};
            for component_expr in components.elems.iter() {
                let component_span = component_expr.span();
                if let Expr::Path(component) = component_expr {
                    let component_expr = create_single_command_stmt(component)?;
                    components_expr = quote_spanned! {component_span=>
                        #components_expr
                        #component_expr
                    };
                } else {
                    throw!(component_span, "Invalid component name")
                }
            }
            components_expr
        }
        _ => throw!(expr.span(), "Invalid components declaration"),
    };
    let expr_span = expr.span();
    Ok(quote_spanned! {expr_span=>
        __ctx.params.add(#core::eml::Param::from_commands("with", ::std::boxed::Box::new(move |c| {
            #with_body
        })));
    })
}

fn create_attr_stmt(ctx: &Context, attr: &NodeAttribute) -> syn::Result<TokenStream> {
    let core = ctx.core_path();
    let attr_name = attr.key.to_string();
    match &attr.value {
        None => {
            return Ok(quote! {
                __ctx.params.add(#core::eml::Param::new(
                    #attr_name.into(),
                    #core::eml::Variant::Bool(true)
                ));
            });
        }
        Some(attr_value) => {
            let attr_value = attr_value.as_ref();
            let attr_span = attr_value.span();
            if attr_name == "with" {
                create_command_stmts(ctx, attr_value)
            } else if attr_name == "params" {
                Ok(quote_spanned! {attr_span=>
                    __ctx.params.merge(#attr_value);
                })
            } else {
                Ok(quote_spanned! {attr_span=>
                    __ctx.params.add(#core::eml::Param::new(
                        #attr_name.into(),
                        (#attr_value).into()
                    ));
                })
            }
        }
    }
}

fn process_for_loop(ctx: &Context, node: &NodeElement) -> syn::Result<TokenStream> {
    let span = node.span();
    if node.attributes.len() != 2 {
        throw!(
            span,
            "<for> tag should have exactly 2 attributes: <for item in=iter>"
        )
    }
    let Node::Attribute(item_attr) = &node.attributes[0] else {
        throw!(span, "Can't threat node ast Node::Attribute")
    };
    if item_attr.value.is_some() {
        throw!(
            span,
            "The first attribute of <for> tag shouldn't has any value: <for item in=iter>"
        )
    }
    let item_ident = Ident::new(&item_attr.key.to_string(), item_attr.span());
    let Node::Attribute(iter_attr) = &node.attributes[1] else {
        throw!(span, "Can't threat node as Node::Attribute")
    };
    if iter_attr.value.is_none() {
        throw!(
            span,
            "The second attribute of <for> tag shold has some value: <for item in=iter>"
        )
    }
    let iter_value = iter_attr.value.as_ref().unwrap().as_ref();

    let mut loop_content = quote! {};
    for ch in node.children.iter() {
        if let Node::Element(elem) = ch {
            if &elem.name.to_string() == "for" {
                let expr = process_for_loop(ctx, elem)?;
                loop_content = quote! {
                    #loop_content
                    #expr
                };
                continue;
            }
        }

        let expr = parse(ctx, ch)?;
        loop_content = quote! {
            #loop_content
            __ctx.children.push( #expr );
        }
    }
    Ok(quote! {
        for #item_ident in #iter_value {
            #loop_content
        }
    })
}

fn process_slots(ctx: &Context, node: &NodeElement) -> syn::Result<TokenStream> {
    let core = ctx.core_path();
    let span = node.span();
    if node.attributes.len() != 1 {
        throw!(
            span,
            "<slot> tag should have exactly 1 attribute: <slot grabber> or <slot name=\"grabber\">"
        )
    }
    let Node::Attribute(attr) = &node.attributes[0] else {
        throw!(span, "Can't threat node ast Node::Attribute")
    };
    let mut slot_content = quote! {};
    for ch in node.children.iter() {
        let expr = parse(ctx, ch)?;
        slot_content = quote! {
            #slot_content
            __slot_value.push( #expr );
        }
    }
    if attr.value.is_none() {
        let slot_name = attr.key.to_string();
        Ok(quote! {
            let mut __slot_value: Vec<Entity> = vec![];
            #slot_content
            __world.resource::<#core::eml::Slots>()
                .clone()
                .insert(#core::tagstr::Tag::new(#slot_name), __slot_value);
        })
    } else {
        if &attr.key.to_string() != "define" {
            throw!(
                span,
                "<slot> definition should have define attribute: <slot define=\"grabber\">"
            )
        }
        let slot_name = attr.value.as_ref().unwrap().as_ref();
        Ok(quote! {
            let __slot_value = __world.resource::<#core::eml::Slots>()
                .clone()
                .remove(#core::tagstr::Tag::new(#slot_name));
            if let Some(__slot_value) = __slot_value {
                __ctx.children.extend(__slot_value);
            } else {
                let mut __slot_value: Vec<Entity> = vec![];
                #slot_content
                __ctx.children.extend(__slot_value);
            }
        })
    }
}

fn parse<'a>(ctx: &Context, element: &'a Node) -> syn::Result<TokenStream> {
    let core = ctx.core_path();
    let mut children = quote! {};
    let mut connections = quote! {};
    let mut parent = quote! {
        let __parent = if __root_builder {
            __parent
        } else {
            __world.spawn_empty().id()
        };
    };
    let Node::Element(element) = element else {
        throw!(element.span(), "Expected eml element")
    };
    let mut parent_defined = false;
    for attr in element.attributes.iter() {
        if let Node::Block(entity) = attr {
            let entity_span = entity.value.span();
            let entity = entity.value.as_ref();
            if parent_defined {
                throw!(entity_span, "Entity already provided by entity attribute")
            }
            parent_defined = true;
            parent = quote! {
                let __parent = #entity;
                if __root_builder {
                    __root_entity_defined = true;
                }
            };
        } else if let Node::Attribute(attr) = attr {
            let attr_name = attr.key.to_string();
            let attr_span = attr.span();
            if let Some(signal) = attr_name.strip_prefix("on:") {
                let Some(handler) = attr.value.as_ref() else {
                    throw!(attr_span, "on:{signal} param should provide connection")
                };
                let signal_ident = syn::Ident::new(signal, handler.span());
                let handler = handler.as_ref();
                let handler_stream = handler.to_token_stream().to_string().trim().to_string();
                let method =
                    if handler_stream.starts_with("run!") || handler_stream.starts_with("run !") {
                        quote! { handle }
                    } else {
                        quote! { func }
                    };
                connections = quote! {
                    #connections
                    __builder.on().#signal_ident().#method(#handler).from(__parent).write(__world);
                }
            } else if let Some(prop) = attr_name.strip_prefix("bind:") {
                let Some(bind) = attr.value.as_ref() else {
                    throw!(attr_span, "bind:{prop} param should provide connection")
                };
                let bind = bind.as_ref();
                let prop = syn::Ident::new(prop, attr.key.span());
                let stream = bind.to_token_stream().to_string();
                if stream.trim().starts_with("to!") || stream.trim().starts_with("to !") {
                    connections = quote_spanned! {attr_span=>
                        #connections
                        (__builder.bind_from().#prop(__parent) >> #bind).write(__world);
                    };
                } else if stream.trim().starts_with("from!") || stream.trim().starts_with("from !")
                {
                    connections = quote_spanned! {attr_span=>
                        #connections
                        (__builder.bind_to().#prop(__parent) << #bind).write(__world);
                    };
                }
            } else if &attr_name == "entity" {
                if parent_defined {
                    throw!(attr_span, "Entity already provided by braced block")
                }
                parent_defined = true;
                let attr_value = attr.value.as_ref();
                if attr_value.is_none() {
                    throw!(attr_span, "Attriute entity should has a value")
                }
                let entity = attr_value.unwrap().as_ref();
                parent = quote_spanned! { attr_span=>
                    let __parent = #entity;
                    if __root_builder {
                        __root_entity_defined = true;
                    }
                };
            } else {
                let attr_stmt = create_attr_stmt(ctx, attr)?;
                children = quote! {
                    #children
                    #attr_stmt
                };
            }
        }
    }
    for child in element.children.iter() {
        match child {
            Node::Element(element) => {
                let element_name = element.name.to_string();
                let expr = match element_name.as_str() {
                    "for" => process_for_loop(ctx, element)?,
                    "slot" => process_slots(ctx, element)?,
                    _ => {
                        let expr = parse(ctx, child)?;
                        quote! {
                            __ctx.children.push( #expr );
                        }
                    }
                };
                children = quote! {
                    #children
                    #expr
                }
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
                        .insert(#core::element::Element::inline())
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
    Ok(quote! {
        {
            #parent
            __root_builder = false;
            let mut __ctx = #core::eml::WidgetData::new(__parent);

            #children
            let __builder = #core::Widgets::#tag();
            __builder.build(__world, __ctx);
            #connections
            __parent
        }
    })
}

pub fn construct(ctx: &Context, root: &Node) -> syn::Result<TokenStream> {
    let body = parse(ctx, root)?;
    let core = ctx.core_path();
    Ok(quote! {
        #core::eml::Eml::new(
            move |
                __world: &mut ::bevy::prelude::World,
                __parent: Option<::bevy::prelude::Entity>,
            | {
                let mut __slots_resource = __world.resource::<#core::eml::Slots>().clone();
                let __defined_slots = __slots_resource.keys();
                let __provided_parent = __parent;
                let __parent = if let Some(parent) = __parent {
                    parent
                } else {
                    __world.spawn_empty().id()
                };
                let mut __root_builder = true;
                let mut __root_entity_defined = false;
                let result = #body;
                for __slot in __slots_resource.keys() {
                    if !__defined_slots.contains(&__slot) {
                        warn!("Detected unused slot '{}', despawning it contnent.", __slot);
                        use ::bevy::ecs::system::Command;
                        for __entity in __slots_resource.remove(__slot).unwrap() {
                            let __despawn =  ::bevy::prelude::DespawnRecursive {
                                entity: __entity
                            };
                            __despawn.write(__world);
                        }
                    }
                }
                if __root_entity_defined {
                    if let Some(parent) = __provided_parent {
                        __world.entity_mut(parent).despawn_recursive();
                    }
                }
                result
            }
        )
    })
}
