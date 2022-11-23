use bevy::prelude::*;
use bevy::reflect::ReflectRef;
use bevy_elements::ess::Stylesheet;
use bevy_elements_core::*;
use bevy_elements_macro::*;
use bevy_elements_widgets::WidgetsPlugin;
use bevy_elements_widgets::input::TextInput;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BsxPlugin)
        .add_plugin(WidgetsPlugin)
        .add_startup_system(setup)
        .add_system(process)
        .run();
    
}

fn setup(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
    commands.add(Stylesheet::parse(r#"
        TextInput:focus .text-input-inner {
            background-color: #efefef;
        }
    "#));
    let time = commands.spawn_empty().insert(TimeSinceStartup::default()).id();
    let left = commands.spawn_empty().id();

    commands.spawn_empty().with_elements(bsx! {
        <el c:ui with=Interaction s:width="100%" s:height="100%">
            <el s:padding="20px" s:align-content="flex-start" s:align-items="flex-start">
                // <TextInput:left value=bind!(<= time, TimeSinceStartup.label) s:margin="8px"/>
                <TextInput:left value="Hello!" s:margin="8px"/>
                <TextInput c:dark value="Help!" s:margin="8px" s:width="100px"/>
            </el>
        </el>
    });
}


#[derive(Reflect, Hash, PartialEq, Eq)]
#[reflect(Hash, PartialEq)]
pub struct E {
    x: i32,
}

fn t() {
    let value: Box<dyn Reflect> = Box::new(E {
        x: 1
    });

    match value.reflect_ref() {
        // `Struct` is a trait automatically implemented for structs that derive Reflect. This trait
        // allows you to interact with fields via their string names or indices
        ReflectRef::Struct(value) => {
            // value
            info!(
                "This is a 'struct' type with an 'x' value of {}",
                value.get_field::<usize>("x").unwrap()
            );
        },
        _ => ()
    }

}


#[derive(Component, Default)]
struct TimeSinceStartup {
    label: String
}


fn process(
    time: Res<Time>,
    mut query: Query<&mut TimeSinceStartup>
) {
    for mut item in query.iter_mut() {
        item.label = format!("Passed: {:.1}0", time.elapsed_seconds());
    }
}