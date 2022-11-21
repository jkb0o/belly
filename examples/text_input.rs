use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy_elements::ess::Stylesheet;
use bevy_elements_core::*;
use bevy_elements_macro::*;
use bevy_elements_widgets::InputPlugins;
use bevy_elements_widgets::input::TextInput;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BsxPlugin)
        .add_plugin(InputPlugins)
        .add_startup_system(setup)
        .add_system(process)
        .run();
    
}

fn setup(
    mut commands: Commands
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.add(Stylesheet::parse(r#"
        text-input:focus .text-input-inner {
            background-color: #efefef;
        }
    "#));
    commands.spawn().with_elements(bsx! {
        <el c:ui with=Interaction s:width="100%" s:height="100%">
            <el s:padding="20px" s:align-content="flex-start" s:align-items="flex-start">
                <text-input value="hello world!" s:margin="8px"/>
                <text-input c:dark value="hello world23!" s:margin="8px" s:width="100px"/>
            </el>
        </el>
    });
}

fn process(
    mut query: Query<&mut TextInput>
) {
    for mut item in query.iter_mut() {
        // item.value = "hello world!!!".to_string();
    }
}