use bevy::prelude::*;
use bevy::reflect::ReflectRef;
use bevy_elements::ess::Stylesheet;
use bevy_elements::input::PointerInput;
use bevy_elements_core::*;
use bevy_elements_macro::*;
use bevy_elements_widgets::WidgetsPlugin;
use bevy_elements_widgets::input::TextInput;
use bevy_elements_widgets::text_line::TextLine;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_plugin(WidgetsPlugin)
        .add_startup_system(setup)
        .add_system(process)
        .add_system(log_events)
        .run();
    
}

fn setup(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
    commands.add(Stylesheet::parse(r#"
        * {
            font: default-regular;
            color: #cfcfcf;
            font-size: 22px;
        }
        .text-input-value {
            color: #2f2f2f;
        }
        .text-input-border:focus {
            background-color: #2f2f2f;
        }
    "#));
    let input = commands.spawn_empty().id();
    commands.add(eml! {
        <el c:ui interactable s:width="100%" s:height="100%" s:padding="20px" s:align-content="flex-start" s:align-items="flex-start">
            <el s:align-content="space-around" s:align-items="center">
                <TextInput {input} value="world" s:margin-right="10px" s:width="100px"/>
                "Hello, "{bind!(<=input, TextInput.value)}"!"
            </el>
        </el>
    });
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

fn log_events(
    mut events: EventReader<PointerInput>
) {
    for e in events.iter().filter(|s| s.pressed() ) {
        info!("signal: {:?}", e);
    }
}