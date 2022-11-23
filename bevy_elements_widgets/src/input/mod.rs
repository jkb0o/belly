use ab_glyph::ScaleFont;
use bevy::{prelude::*, ui::FocusPolicy, text::TextLayoutInfo};
use bevy_elements_core::{*, element::{Element, DisplayElement}, builders::DefaultFont, attributes::AttributeValue};
use bevy_elements_macro::*;
use itertools::Itertools;
pub struct InputPlugins;
use crate::text_line::TextLine;

const CHAR_DELETE: char = '\u{7f}';

impl Plugin for InputPlugins {
    fn build(&self, app: &mut App) {
        app.register_element_builder("text", build_text);
        app.add_system(blink_cursor);
        app.add_system(process_cursor_focus);
        app.add_system(process_keyboard_input);
    }
}

#[derive(Component)]
pub struct TextInput {
    pub value: String,
    index: usize,
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
    let cursor = commands.spawn_empty().id();
    let text = commands.spawn_empty().id();
    let block_input = FocusPolicy::Block;
    let widget = TextInput {
        cursor, text,
        index: 0,
        value: value.unwrap_or("".to_string()),
    };
    commands.entity(entity).with_elements(bsx! {
        <el with=(widget,block_input,BackgroundColor,Interaction) c:text-input s:background-color="#2f2f2f" s:padding="1px">
            <el 
                with=BackgroundColor
                c:text-input-inner 
                s:width="100%"
                s:heigth="100%"
                s:background-color="#cfcfcf"
                s:overflow="hidden"
                // s:height="24px"
                s:width="100%"
            >
                // <el c:text-input-selection/>
                <TextLine:text value=bind!(<= entity, Self.value) s:color="#2f2f2f" s:margin-left="2px"/>
                <el:cursor
                    with=BackgroundColor
                    c:text-input-cursor
                    s:position-type="absolute"
                    s:top="1px"
                    s:bottom="1px"
                    s:left="1px"
                    // s:right="2px"
                    s:width="2px"
                    s:display="none"
                    // s:height="20px"
                    s:background-color="#2f2f2f"
                />
            </el>
        </el>
    });
});



fn get_char_advance(
    ch: char,
    font: &Font,
    font_size: f32,
) -> f32 {
    let font = ab_glyph::Font::as_scaled(&font.font, font_size);
    let glyph = font.glyph_id(ch);
    font.h_advance(glyph)
}

fn process_keyboard_input(
    changed_elements: Query<(), Changed<Element>>,
    changed_layout: Query<(), Changed<TextLayoutInfo>>,
    keyboard: Res<Input<KeyCode>>,
    fonts: Res<Assets<Font>>,
    mut characters: EventReader<ReceivedCharacter>,
    mut inputs: Query<(Entity, &mut TextInput, &Element)>,
    mut cursors: Query<(&mut TextInputCursor, &mut Style)>,
    mut texts: Query<&TextLine>,
) {
    let Some((entity, mut input)) = inputs.iter_mut()
        .filter(|(_, _, e)| e.focused())
        .map(|(e, i, _)| (e, i))
        .next() 
        else { return };
    let controls_pressed = keyboard.any_just_pressed([
        KeyCode::Left,
        KeyCode::Right
    ]);
    if characters.is_empty() 
    && !controls_pressed 
    && !changed_elements.contains(entity)
    && !changed_layout.contains(input.text) {
        return;
    }
    let Ok((mut cursor, mut style)) = cursors.get_mut(input.cursor) 
        else { return };
    let Ok(text) = texts.get_mut(input.text)
        else { return };

    let mut chars: Vec<_> = input.value.chars().collect();
    if keyboard.just_pressed(KeyCode::Left) {
        if input.index > 0 {
            input.index -= 1;
        }
    } else if keyboard.just_pressed(KeyCode::Right) {
        if input.index < chars.len() {
            input.index += 1;
        }
    } else { for ch in characters.iter() {
        if ch.char == CHAR_DELETE {
            if input.index > 0 {
                let idx = input.index - 1;
                chars.remove(idx);
                input.value = chars.iter().collect();
                input.index = idx;
            }
        } else {
            let idx = input.index;
            chars.insert(idx, ch.char);
            input.value = chars.iter().collect();
            input.index += 1;
        }
    }}

    cursor.state = 1.;
    let mut pos = 2.;
    let Some(font) = fonts.get(&text.style.font) else { return };
    let font_size = text.style.font_size;
    for (idx, ch) in chars.iter().enumerate() {
        if idx == input.index {
            break;
        }
        pos += get_char_advance(*ch, font, font_size);
    }
    style.position.left = Val::Px(pos);
    
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
        .insert(TextBundle::from_section(
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