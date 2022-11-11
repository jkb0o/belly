use bml_core::*;
use bsx::*;
use bevy::prelude::*;

// fn build_window(In(ctx): In<Context>, mut commands: Commands) {
//     commands.entity(ctx.element).with_elements(bsx! {
//         <box>
//             <box>"Header"</box>
//             <box>
//                 "Content"
//                 <box>
//                 { ctx.child_elements }
//                 </box>
//             </box>
//         </box>
//     });
// }

fn test(mut commands: Commands) {
    commands.spawn().with_elements(bsx!{
        <el>"Hello world!"</el>
    });
}
// use std::prelude::rust_2021::*;
// #[macro_use]
// extern crate std;
// use bml_core::*;
// use bsx::*;
// use bevy::prelude::*;
// fn build_window(In(ctx): In<Context>, mut commands: Commands) {
//     commands
//         .add(
//             ::bml_core::SceneBuilder::new(|
//                 __world: &mut ::bevy::prelude::World,
//                 __parent: ::bevy::prelude::Entity|
//             {
//                 {
//                     let mut __ctx = ::bml_core::Context::new(__parent);
//                     __ctx
//                         .child_elements
//                         .push({
//                             let __parent = __world.spawn().id();
//                             let mut __ctx = ::bml_core::Context::new(__parent);
//                             {
//                                 let __text_entity = __world.spawn().id();
//                                 __ctx.child_elements.push(__text_entity.clone());
//                                 let __text_ctx = ::bml_core::Context::new(__text_entity);
//                                 __world
//                                     .resource::<::bml_core::TextElementBuilder>().clone()
//                                     .build(__text_ctx, "Header".to_string(), __world);
                                    
//                             }
//                             __world
//                                 .resource::<::bml_core::ElementBuilderRegistry>()
//                                 .get_builder("box")
//                                 .expect("Invalid tag name: box")
//                                 .build(__ctx, __world);
                            
//                             __parent
//                         });
//                     __world
//                         .resource::<::bml_core::ElementBuilderRegistry>()
//                         .get_builder("box")
//                         .expect("Invalid tag name: box")
//                         .build(__ctx, __world);
//                     __parent
//                 };
//             }),
//         )
// }


