// use bevy::{prelude::*, ecs::system::SystemParam};
// pub struct Context {
//     pub name: &'static str,
//     pub element: Entity,
//     pub child_elements: Vec<Entity>,
// }

// impl Context {
//     pub fn new(name: &'static str, element: Entity) -> Context {
//         Context {
//             name, element,
//             child_elements: vec![],
//         }
//     }
// }

// fn build_stuff(In(_): In<Context>) {
//     ();
// }

// fn register_stuff_builder<Param, Marker, Func>(func: Func, world: &mut World)
//     where 
//         Func: SystemParamFunction<Context, (), Param, Marker>,
//         Param: SystemParam + 'static,
//         Marker: 'static,
// {
//     let entity = world.spawn().id();
//     let ctx = Context::new("a", entity);
//     // For the real world I store this system as 
//     // resource into world and use it later
//     let mut system = IntoSystem::into_system(func);
//     system.run(ctx, world);
// }

// fn setup_things(mut commands: Commands) {
//     commands.add(|world: &mut World| {
//         register_stuff_builder(build_stuff, world);
//     });
// }

// fn main() {
//     App::new()
//         .add_system(setup_things);
// }

use std::sync::Once;

use bevy::{
    prelude::*
};
use bevy_bsx::*;
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(BsxPlugin)
        .add_startup_system(setup)
        .register_element_builder("ui", build_ui)
        .register_element_builder("hbox", build_hbox)
        .register_element_builder("vbox", build_vbox)
        .register_element_builder("window", build_window)
        .run();
}

fn build_ui(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
    let mut elem = commands.entity(ctx.element);
    elem.insert_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            padding: UiRect::all(Val::Px(20.)),
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            flex_direction: FlexDirection::ColumnReverse,
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            align_self: AlignSelf::Center,  
            ..default()
        },
        ..default()
    }).with_children(|parent|{
        parent.spawn().insert_bundle(NodeBundle {
            color: UiColor(Color::rgba(0.2, 0.2, 0.2, 0.2)),
            style: Style {
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        }).push_children(&ctx.content());
    });

}

fn build_vbox(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
    let mut elem = commands.entity(ctx.element);
    elem.insert_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::ColumnReverse,
            ..default()
        },
        ..default()
    }).push_children(&ctx.content());

}

fn build_hbox(    
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
let mut elem = commands.entity(ctx.element);
elem.insert_bundle(NodeBundle {
    color: UiColor(Color::NONE),
    style: Style {
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Row,
        ..default()
    },
    ..default()
}).push_children(&ctx.content());

}

fn build_window(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
    let content = ctx.content();
    commands.entity(ctx.element).with_elements(bsx! {
        <vbox class="window" with>
            <el class="window-header">
                <el class="window-header-text">
                    "Window"
                </el>
                <el class="window-header-close-btn"/>
            </el>
            <el class="window-content">
                {content}
            </el>
        </vbox>
    });
}


fn setup(
    mut commands: Commands,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    // let x = bsx! { };

    commands.spawn().with_elements(bsx! {
        <ui>
            <window 
                id="hello" 
                class="a b c"
                c:active c:light 
                title="I'm a window!" 
                int=23 
                // with=(1,2,3)
            >
                "hello world!"
            </window>
        </ui>
    });
    // commands.add(|world: &mut World| {
        
    //     let asset_server = world.get_resource::<AssetServer>().unwrap().clone();
    //     let mut entity = world.spawn();
    //     entity.insert_bundle(TextBundle::from_section(
    //         "Hello world".to_string(),
    //         TextStyle {
    //             font: asset_server.load("FiraMono-Medium.ttf"),
    //             font_size: 50.0,
    //             color: Color::WHITE,
    //         },
    //     ));
    // })
    // spawn(&mut commands, &asset_server);


}

fn test() -> Tag {
    unsafe { 
        static mut tag: Tag;
        static once: Once = Once::new();
        once.call_once(|| tag = bevy_bsx::tags::tag("name")); 
        tag
    }
}