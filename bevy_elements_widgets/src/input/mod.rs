use bevy::{prelude::*, ui::FocusPolicy, text::TextLayoutInfo};
use bevy_elements_core::{*, element::{Element, DisplayElement}, builders::DefaultFont, attributes::AttributeValue};
use bevy_elements_macro::*;

pub struct InputPlugins;

impl Plugin for InputPlugins {
    fn build(&self, app: &mut App) {
        app.register_element_builder("text", build_text);
        app.add_system(blink_cursor);
        app.add_system(process_cursor_focus);
        // app.register_element_builder("textinput", build_text_input);
    }
}

#[derive(Component)]
pub struct TextInput {
    pub value: String,
    cursor: Entity,
    text: Entity
}

#[derive(Component, Default)]
pub struct TextInputCursor {
    state: f32
}


widget!( TextInput,
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
=> {
    let entity = ctx.element;
    let value = bindattr!(ctx, commands, value:String => Self.value);
    let cursor = commands.spawn().id();
    let text = commands.spawn().id();
    let block_input = FocusPolicy::Block;
    let widget = TextInput {
        cursor, text,
        value: value.unwrap_or("".to_string()),
    };
    commands.entity(entity).with_elements(bsx! {
        <el with=(widget,block_input,UiColor,Interaction) c:text-input s:background-color="#2f2f2f" s:padding="1px">
            <el 
                with=UiColor
                c:text-input-inner 
                s:width="100%"
                s:heigth="100%"
                s:background-color="#cfcfcf"
                s:overflow="hidden"
                s:height="24px"
                s:width="100%"
            >
                // <el c:text-input-selection/>
                <text:text value=bind!(<= entity, Self.value) s:color="#2f2f2f" s:margin-left="2px"/>
                <el:cursor
                    with=UiColor
                    c:text-input-cursor
                    s:position-type="absolute"
                    s:top="1px"
                    s:bottom="1px"
                    s:left="1px"
                    // s:right="2px"
                    s:width="1px"
                    s:display="none"
                    // s:height="20px"
                    s:background-color="#2f2f2f"
                />
            </el>
        </el>
    });
});

fn process_keyboard_input(
    mut inputs: Query<(&mut TextInput, &Element)>,
    mut cursors: Query<(&mut TextInputCursor, &mut Style)>,
    mut texts: Query<(&mut Text, &TextLayoutInfo)>,

) {
    let input = inputs.iter_mut()
        .filter(|(_, e)| e.focused())
        .map(|(i, _)| i)
        .next();
    let mut input = if let Some(input) = input {
        input
    } else {
        return
    };





}

fn process_cursor_focus(
    mut commands: Commands,
    input: Query<(&TextInput, &Element), Changed<Element>>,
    cursors: Query<&TextInputCursor>,
    mut styles: Query<&mut Style>,
) {
    for (input, element) in input.iter() {
        if element.focused() && !cursors.contains(input.cursor) {
            commands.entity(input.cursor).insert(TextInputCursor::default());
        }
        if !element.focused() && cursors.contains(input.cursor) {
            commands.entity(input.cursor).remove::<TextInputCursor>();
            if let Ok(mut style) = styles.get_mut(input.cursor) {
                style.display = Display::None;
            }
        }
    }
}

fn blink_cursor(
    time: Res<Time>,
    mut cursor: Query<(&mut TextInputCursor, &mut Style)>,
) {
    for (mut cursor, mut style) in cursor.iter_mut() {
        cursor.state -= time.delta_seconds();
        if cursor.state < 0. {
            cursor.state = 1.;
        }
        if cursor.state >= 0.5 && style.display == Display::None {
            style.display = Display::Flex;
        }
        if cursor.state < 0.5 && style.display != Display::None {
            style.display = Display::None;
        }
    }

}

fn build_text(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
    default_font: Res<DefaultFont>,
) {
    let text = bindattr!(ctx, commands, value:String => Text.sections[0].value);
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