use proc_macro2::TokenStream;
use quote::*;
extern crate proc_macro;
use syn_rsx::{Node, NodeType, parse};

fn walk_nodes<'a>(element: &'a Node, create_entity: bool) -> TokenStream {
    let mut children = quote! { };
    for attr in element.attributes.iter() {
        let attr_name = attr.name_as_string().expect("Invalid attribute name");
        match attr.value {
            None => {
                children = quote! {
                    #children
                    __ctx.attributes.add(::bml_core::attributes::Attribute::new(
                        #attr_name.into(),
                        ::bml_core::attributes::AttributeValue::Empty
                    ));
                }
            }
            Some(ref attr_value) => {
                children = quote! {
                    #children
                    __ctx.attributes.add(::bml_core::attributes::Attribute::new(
                        #attr_name.into(),
                        (#attr_value).into()
                    ));
                }

            }
        }
    }
    for child in element.children.iter() {
        match child.node_type {
            NodeType::Element => {
                let expr = walk_nodes(child, true);
                children = quote! {
                    #children
                    __ctx.add_child( #expr );
                };
            },
            NodeType::Text => {
                let text = child.value.as_ref().expect("Can't fetch the text");
                children = quote! {
                    #children
                    {
                        let __text_entity = __world.spawn().id();
                        __ctx.add_child(__text_entity.clone());
                        ::bml_core::context::internal::push_text(__world, __text_entity, #text.to_string());
                        __world
                            .resource::<::bml_core::builders::TextElementBuilder>().clone()
                            .build(__world);
                        ::bml_core::context::internal::pop_context(__world);
                    }
                };
            },
            NodeType::Block => {
                let block = child.value.as_ref().unwrap();
                children = quote! {
                    #children
                    for __child in #block.iter() {
                        __ctx.add_child( __child.clone() );
                    }
                }
            }
            _ => ()
        };
    }

    let tag = element.name_as_string().expect("Element withot name");
    let invalid_element_msg = format!("Invalid tag name: {}", tag);
    let parent = if create_entity {
        quote! { let __parent = __world.spawn().id(); }
    } else {
        quote! { }
    };
    quote! {
        {
            #parent
            let __tag_name = #tag.into();
            let mut __ctx = ::bml_core::context::ElementContext::new(__tag_name, __parent);
            
            #children
            
            ::bml_core::context::internal::push_element(__world, __ctx);
            __world
                .resource::<::bml_core::builders::ElementBuilderRegistry>()
                .get_builder(__tag_name)
                .expect( #invalid_element_msg )
                .build(__world);
            ::bml_core::context::internal::pop_context(__world);
            __parent
        }
    }
}


#[proc_macro]
pub fn bsx(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(tree.into()) {
        Err(err) => err.to_compile_error().into(),
        Ok(nodes) => {
            let body = walk_nodes(&nodes[0], false);
            // nodes[0]
            let wraped = quote! {
                ::bml_core::ElementsBuilder::new(
                    |
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

// #macro_rules!  {
//     () => {
        
//     };
// }