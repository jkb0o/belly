use std::sync::Once;

use bevy::{
    prelude::*, ecs::{schedule::IntoSystemDescriptor, system::{EntityCommands, BoxedSystem}}
};
use bevy_elements::{*, ess::Stylesheet};
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
        .add_system(test_system)
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
    let content = ctx.content();
    commands.entity(ctx.element).with_elements(bsx! {
        <el s:justify-content="center" s:flex-direction="column-reverse">
            {content}
        </el>
    });
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
    let header = ctx.param("title", "Title".to_string());
    commands.entity(ctx.element).with_elements(bsx! {
        <vbox class="window" c:cool-window s:background-color="palevioletred">
            <el class="window-header">
                <el class="window-header-text">
                    {header}
                </el>
                <el class="window-header-close-btn"/>
            </el>
            <el class="window-content">
                {content}
            </el>
        </vbox>
    });
}

#[derive(Component, Default)]
struct Test;

fn custom_builder() -> ElementsBuilder {
    bsx! { <el/> }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.add(Stylesheet::parse(r#"
        .winxxx {
            padding-left: 20px;
            margin-left: 20px;
        }
    "#));
    // let x = bsx! { };

    let transform = Transform::default();
    let elements = &["a", "b"];

    commands.spawn().with_elements(bsx! {
        <ui>
            <window title="I'm a window!" c:win s:height="400px" s:width="300px" with=(transform,Test)>
                <vbox>
                    "hello world!"
                    {elements.iter().elements(|e| { bsx! {
                        <el>{e.to_string()}</el> 
                    }})}
                </vbox>
            </window>
        </ui>
    });
    let system: BoxedSystem<_, _> = Box::new(IntoSystem::into_system(custom_builder));
    // system.into_content();
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

fn test() {
    test2(|| {});
}

fn test2<Params, F: IntoSystem<(), (), Params>>(f: F) {
    use bevy_elements_core::attributes::*;
    let a = IntoAttr::into_attr("hello".to_string());
    let b = IntoAttr::into_attr(f);
    let c = IntoAttr::into_attr(2);
    let f = |c: &mut EntityCommands| c.despawn();
    // let boxed = Box::new(|c: &mut EntityCommands| c.despawn());
    // let d = IntoAttr::into_attr(f.into());
    // let x = components!(Transform);
    // let a = vec![];

    // let a = ["a", "b"].iter().elements(|e| bsx! { <el>{e.to_string()}</el> }).into_content(world)

}

fn test_system(
    mut commands: Commands,
    q: Query<Entity, With<Test>>
) {
    return;
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}