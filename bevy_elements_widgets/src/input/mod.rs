use ab_glyph::ScaleFont;
use bevy::{prelude::*, ui::FocusPolicy, input::keyboard::KeyboardInput, diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}};
use bevy_elements_core::{*, element::{Element, DisplayElement}, builders::DefaultFont, signals::Signal};
use bevy_elements_macro::*;
pub struct InputPlugins;
use crate::text_line::TextLine;

const CHAR_DELETE: char = '\u{7f}';
const CURSOR_WIDTH: f32 = 2.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum TextInputLabel {
    Focus,
    Mouse,
    Keyboard
}

impl Plugin for InputPlugins {
    fn build(&self, app: &mut App) {
        app
            .register_element_builder("text", build_text)
            .add_system(blink_cursor)
            .add_system_to_stage(CoreStage::PreUpdate, process_cursor_focus
                .label(TextInputLabel::Focus)
                .after(ElementsLabel::Focus)
            )
            .add_system_to_stage(CoreStage::PreUpdate, process_mouse
                .label(TextInputLabel::Mouse)
                .after(TextInputLabel::Focus)
                // .after(TextInputLabel::Focus)
            )
            .add_system_to_stage(CoreStage::PreUpdate, process_keyboard_input
                .label(TextInputLabel::Keyboard)
                .after(TextInputLabel::Mouse)
            )
            ;
    }
}

#[derive(Component)]
pub struct TextInput {
    pub value: String,
    index: usize,
    cursor: Entity,
    text: Entity,
    container: Entity,
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
    let container = commands.spawn_empty().id();
    let block_input = FocusPolicy::Block;
    let widget = TextInput {
        cursor, text, container,
        index: 0,
        value: value.unwrap_or("".to_string()),
    };
    commands.entity(entity).with_elements(bsx! {
        <el with=(widget,block_input,BackgroundColor,Interaction) 
            c:text-input 
            s:background-color="#2f2f2f" 
            s:padding="1px"
            s:width="200px"
        >
            <el with=BackgroundColor
                c:text-input-background
                s:padding="1px"
                s:width="100%"
                s:height="100%"
                s:background-color="#efefef"
            >
                <el {container}
                    c:text-input-container
                    s:width="100%"
                    s:heigth="100%"
                    s:width="100%"
                    s:overflow="hidden"
                >
                    <TextLine {text} value=bind!(<= entity, Self.value) s:color="#2f2f2f"/>
                    <el entity=cursor
                        with=BackgroundColor
                        c:text-input-cursor
                        s:position-type="absolute"
                        s:top="1px"
                        s:bottom="1px"
                        // s:left="1px"
                        // s:right="2px"
                        s:width=format!("{:.0}px", CURSOR_WIDTH)
                        s:display="none"
                        // s:height="20px"
                        s:background-color="#2f2f2f"
                    />
                </el>
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
    keyboard_input: EventReader<KeyboardInput>,
    keyboard: Res<Input<KeyCode>>,
    fonts: Res<Assets<Font>>,
    nodes: Query<&Node>,
    mut characters: EventReader<ReceivedCharacter>,
    mut inputs: Query<(Entity, &mut TextInput, &Element)>,
    mut cursors: Query<&mut TextInputCursor>,
    mut styles: Query<&mut Style>,
    mut texts: Query<&TextLine>,
    diag: Res<Diagnostics>,
) {
    let frame = diag.get(FrameTimeDiagnosticsPlugin::FRAME_COUNT).unwrap().average().unwrap_or_default();
    let Some((entity, mut input)) = inputs.iter_mut()
        .filter(|(_, _, e)| e.focused())
        .map(|(e, i, _)| (e, i))
        .next() 
        else { return };
    if characters.is_empty()
    && keyboard_input.is_empty()
    && !changed_elements.contains(entity) {
        return;
    }
    
    let Ok(text) = texts.get_mut(input.text)
        else { return };
    
    let mut index = input.index;

    let mut chars: Vec<_> = input.value.chars().collect();
    if keyboard.just_pressed(KeyCode::Left) {
        if index > 0 {
            index -= 1;
        }
    } else if keyboard.just_pressed(KeyCode::Right) {
        if index < chars.len() {
            index += 1;
        }
    } else { for ch in characters.iter() {
        if ch.char == CHAR_DELETE {
            if index > 0 {
                let idx = index - 1;
                chars.remove(idx);
                input.value = chars.iter().collect();
                index = idx;
            }
        } else {
            let idx = index;
            chars.insert(idx, ch.char);
            input.value = chars.iter().collect();
            index += 1;
        }
    }}

    if let Ok(mut cursor) = cursors.get_mut(input.cursor) {
        cursor.state = 1.;
    }
    let Ok(node) = nodes.get(input.container) else { return };
    let container_width = node.size().x;
    let mut position_from_start = 0.;
    let mut text_width = 0.;
    let Some(font) = fonts.get(&text.style.font) else { return };
    let font_size = text.style.font_size;
    for (idx, ch) in chars.iter().enumerate() {
        let advance = get_char_advance(*ch, font, font_size);
        text_width += advance;
        if idx < index {
            position_from_start += advance;
        }
    }
    let mut offset = if let Ok(contaienr_style) = styles.get_mut(input.container) {
        match contaienr_style.padding.left {
            Val::Px(x) => x,
            _ => 0.
        }
    } else {
        0.
    };
    if offset + position_from_start < 0. {
       offset = -position_from_start;
    }
    if offset + position_from_start > container_width - CURSOR_WIDTH{
        offset = container_width - position_from_start - CURSOR_WIDTH;
    }
    let unused_space = container_width - text_width - offset - CURSOR_WIDTH;
    if unused_space > 0. {
        offset = (offset + unused_space).min(0.);
    }
    let cursor_position = position_from_start + offset;
    // let offset = (position_from_start - container_width).max(0.);
    if let Ok(mut cursor_style) = styles.get_mut(input.cursor) {
        cursor_style.position.left = Val::Px(cursor_position);
    }
    if let Ok(mut contaienr_style) = styles.get_mut(input.container) {
        contaienr_style.padding.left = Val::Px(offset);
    }
    if input.index != index {
        input.index = index;
    }
    
    info!("{}:process_keyboard_input: Resulting index: {}, cursor: {}",  frame, input.index, cursor_position);
    
}

fn process_cursor_focus(
    mut commands: Commands,
    mut input: Query<(&mut TextInput, &Element), Changed<Element>>,
    cursors: Query<&TextInputCursor>,
    mut styles: Query<&mut Style>,
) {
    for (mut input, element) in input.iter_mut() {
        if element.focused() && !cursors.contains(input.cursor) {
            commands.entity(input.cursor).insert(TextInputCursor::default());
        }
        if !element.focused() && cursors.contains(input.cursor) {
            input.index = 0;
            commands.entity(input.cursor).remove::<TextInputCursor>();
            if let Ok(mut style) = styles.get_mut(input.cursor) {
                style.display = Display::None;
            }
            if let Ok(mut contaienr_style) = styles.get_mut(input.container) {
                contaienr_style.padding.left = Val::Px(0.);
            }
        }
    }
}

fn process_mouse(
    mut events: EventReader<Signal>,
    mut inputs: Query<(Entity, &mut TextInput, &mut Element)>,
    texts: Query<&TextLine>,
    styles: Query<(&Style, &GlobalTransform, &Node)>,
    fonts: Res<Assets<Font>>,
    diag: Res<Diagnostics>,
) {
    for evt in events.iter().filter(|s| s.pressed()) {
        for (entity, mut input, mut element) in inputs.iter_mut() {
            if !evt.contains(entity) {
                continue;
            }
            let Ok((container, tr, node)) = styles.get(input.container) else { continue };
            let mut offset = if let Val::Px(offset) = container.padding.left {
                offset
            } else {
                0.
            };
            let Ok(text) = texts.get(input.text) else { continue };
            let Some(font) = fonts.get(&text.style.font) else { continue };
            let font_size = text.style.font_size;
            let pressed = (evt.pos - tr.translation().truncate() + node.size() * 0.5).x;
            let mut idx = 0;
            for ch in text.value.chars() {
                let advance = get_char_advance(ch, font, font_size);
                if offset + advance * 0.5 > pressed {
                    break;
                }
                offset += advance;
                idx += 1;
            }
            if input.index != idx {
                input.index = idx;
                element.invalidate();
            }
            let frame = diag.get(FrameTimeDiagnosticsPlugin::FRAME_COUNT).unwrap().average().unwrap_or_default();
            info!("{}:process_mouse: Clicked relative: {:.2}, idx={}, offset={}, focused: {}", frame, pressed, idx, offset, element.focused());

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