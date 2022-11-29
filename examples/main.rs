
use bevy::{
    prelude::*, input::keyboard::KeyboardInput
};
use bevy_elements::*;
use bevy_elements::build::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .register_element_builder("ui", build_ui)
        .register_element_builder("hbox", build_hbox)
        .register_element_builder("vbox", build_vbox)
        .register_element_builder("window", build_window)
        .add_system(print_char_system)
        .run();
}

fn build_ui(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
    let mut elem = commands.entity(ctx.element);
    elem.insert(NodeBundle {
        background_color: BackgroundColor(Color::NONE),
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
        parent.spawn(NodeBundle {
            background_color: BackgroundColor(Color::rgba(0.2, 0.2, 0.2, 0.2)),
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
    commands.entity(ctx.element).with_elements(eml! {
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
elem.insert(NodeBundle {
    background_color: BackgroundColor(Color::NONE),
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
    commands.entity(ctx.element).with_elements(eml! {
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

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::parse(r#"
        .winxxx {
            padding-left: 20px;
            margin-left: 20px;
        }
    "#));

    let transform = Transform::default();
    let elements = &["a", "b"];

    commands.spawn_empty().with_elements(eml! {
        <ui>
            <window title="I'm a window!" c:win s:height="400px" s:width="300px" with=(transform,Test)>
                <vbox>
                    "hello world!"
                    {elements.iter().elements(|e| { eml! {
                        <el>{e.to_string()}</el> 
                    }})}
                </vbox>
            </window>
        </ui>
    });
}

fn print_char_system(
    mut chr: EventReader<ReceivedCharacter>,
    mut kbd: EventReader<KeyboardInput>,
    mut q_text: Query<&mut Text>,
    keys: Res<Input<KeyCode>>,
) {
    for e in kbd.iter() {
        println!("kbd: {:?}, {:?}", e.key_code, e.scan_code);
        
    }
    let cmd_key = keys.pressed(KeyCode::LWin) 
            || keys.pressed(KeyCode::RWin)
            || keys.pressed(KeyCode::LControl)
            || keys.pressed(KeyCode::RControl);
    if cmd_key {
        return;
    }
    for e in chr.iter() {
        // let cmd_key = keys.pressed(KeyCode::LWin) 
        //     || keys.pressed(KeyCode::RWin)
        //     || keys.pressed(KeyCode::LControl)
        //     || keys.pressed(KeyCode::RControl);
        //     // || keys.pressed(KeyCode::L)
        // println!("chr: {}, {:?}", e.char, cmd_key);
        for mut text in q_text.iter_mut() { 
            if keys.pressed(KeyCode::Back) {
                text.sections[0].value.pop();
                continue;
            }
            text.sections[0].value.push(e.char);
        }
    }
}