use bevy::prelude::*;
use bevy_elements::build::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .register_element_builder("body", build_body)
        .register_element_builder("div", build_div)
        .run();
    
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::parse(r#"
        .text-input-value {
            color: #2f2f2f;
        }
        .text-input-border:focus {
            background-color: #2f2f2f;
        }
        .center-left {
            align-content: space-around;
            align-items: center;
        }
    "#));
    let input = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="20px">
            <div c:center-left>
                <TextInput {input} value="world" s:margin-right="10px" s:width="100px"/>
                "Hello, "{bind!(<=input, TextInput.value)}"!"
            </div>
        </body>
    });
}

fn build_body(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
    let content = ctx.content();
    commands.entity(ctx.element).with_elements(eml! {
        <el interactable s:width="100%" s:height="100%" s:align-content="flex-start" s:align-items="flex-start">
            {content}
        </el>
    });
}

fn build_div(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
) {
    let content = ctx.content();
    commands.entity(ctx.element).with_elements(eml! {
        <el>{content}</el>
    });
}