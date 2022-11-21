use bevy::{prelude::*, ui::FocusPolicy};
use bevy_elements_core::{*, element::{Element, DisplayElement}, builders::DefaultFont, attributes::AttributeValue};
use bevy_elements_macro::*;

pub struct InputPlugins;

impl Plugin for InputPlugins {
    fn build(&self, app: &mut App) {
        app.register_element_builder("text", build_text);
        app.register_element_builder("text-input", build_text_input);
    }
}

#[derive(Component)]
pub struct TextInput {
    pub value: String,
    cursor: Entity
}


fn build_text_input(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
) {
    let entity = ctx.element;
    let t = ctx.attributes.get::<String>("value".as_tag());
    info!("got text for input 0: {:?}", t);
    let value = bindattr!(ctx, commands, value:String => TextInput.value);
    info!("got text for input 1: {:?}", value);
    let cursor = commands.spawn().id();
    let block = FocusPolicy::Block;
    let widget = TextInput {
        cursor,
        value: value.unwrap_or("".to_string()),
    };
    commands.entity(entity).with_elements(bsx! {
        <el with=(widget,block,UiColor,Interaction) c:text-input s:background-color="#2f2f2f" s:padding="1px">
            <el with=UiColor c:text-input-inner 
                s:width="100%"
                s:heigth="100%"
                s:background-color="#cfcfcf"
                s:overflow="hidden"
                s:height="24px"
                s:width="100%"
            >
                <el c:text-input-selection/>
                <text value=bind!(<= entity, TextInput.value) s:color="#2f2f2f"/>
                <el:cursor c:text-input-cursor/>
            </el>
        </el>
    });
}

fn build_text(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
    default_font: Res<DefaultFont>,
) {
    let t = ctx.attributes.get_variant("value".as_tag());
    info!("got text 0: {:?}", t);
    let text = bindattr!(ctx, commands, value:String => Text.sections[0].value);
    info!("got text: {:?}", text);
    commands
        .entity(ctx.element)
        .insert_bundle(TextBundle::from_section(
            // "hahaha".to_string(),
            text.unwrap_or("".to_string()),
            TextStyle {
                font: default_font.0.clone(),
                font_size: 24.0,
                color: Color::WHITE,
            },
        ))
        .insert(Element {
            display: DisplayElement::Inline,
            ..default()
        });
}