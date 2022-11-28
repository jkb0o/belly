use std::ops::Range;

use ab_glyph::ScaleFont;
use bevy::{prelude::*, ui::FocusPolicy, input::keyboard::KeyboardInput, diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}};
use bevy_elements_core::{*, element::{Element, DisplayElement}, builders::DefaultFont, input::PointerInput};
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
                .after(bevy_elements_core::input::Label::Focus)
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
    selected: Selection,
    cursor: Entity,
    text: Entity,
    container: Entity,
    selection: Entity,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Selection {
    min: usize,
    max: usize,
    started: Option<usize>
}

impl Selection {
    pub fn new() -> Selection {
        Selection {
            min: 0,
            max: 0,
            started: None
        }
    }
    pub fn is_empty(&self) -> bool {
        self.max == self.min
    }

    pub fn size(&self) -> usize {
        self.max - self.min
    }

    pub fn start(&mut self, value: usize) {
        self.started = Some(value);
        self.min = value;
        self.max = value;
    }

    pub fn stop(&mut self) {
        self.started = None;
        self.min = 0;
        self.max = 0;
    }

    pub fn extend(&mut self, value: usize) {
        if self.started.is_none() {
            self.start(value);
            return;
        }
        let started = self.started.unwrap();
        if value > self.max {
            self.max = value;
            self.min = started;
        } else if  value < self.min {
            self.min = value;
            self.max = started;
        } else if value > started {
            self.max = value;
        } else if value < started {
            self.min = value;
        } else {
            self.min = started;
            self.max = started;
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.min..self.max
    }
}

// impl From<usize> for Selection {
//     fn from(idx: usize) -> Self {
//         Self::new(idx)
//     }
// }

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
    let selection = commands.spawn_empty().id();
    // let block_input = FocusPolicy::Block;
    let widget = TextInput {
        cursor, text, container, selection,
        index: 0, selected: Selection::new(),
        value: value.unwrap_or("".to_string()),
    };
    commands.entity(entity).with_elements(eml! {
        <el with=widget 
            interactable="block"
            c:text-input 
            c:text-input-border
            s:background-color="#2f2f2f00" 
            s:padding="2px"
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
                    <el {selection} with=BackgroundColor
                        c:text-input-selection
                        s:position-type="absolute"
                        s:height="100%"
                        s:display="none"
                        // s:left="0px"
                        // s:width="50px"
                        s:background-color="#9f9f9f"
                    />

                    <TextLine {text} value=bind!(<= entity, Self.value) s:color="#2f2f2f" c:text-input-value/>
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
    mut texts: Query<(&TextLine, &Text)>,
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
    
    let Ok((text_line, text)) = texts.get_mut(input.text)
        else { return };
    
    let shift = keyboard.any_pressed([KeyCode::LShift, KeyCode::RShift]);
    let mut index = input.index;
    let mut selected = input.selected.clone();

    let mut chars: Vec<_> = input.value.chars().collect();
    if keyboard.just_pressed(KeyCode::Left) {
        if !shift {
            selected.stop();
        }
        if index > 0 {
            index -= 1;
            if shift { 
                selected.extend(index + 1);
                selected.extend(index);
            }
        }
    } else if keyboard.just_pressed(KeyCode::Right) {
        if !shift {
            selected.stop();
        }
        if index < chars.len() {
            index += 1;
            if shift { 
                selected.extend(index - 1);
                selected.extend(index); 
            }
        }
    // } else if keyboard.just_pressed(KeyCode::Tab) {
        // continue
    } else { for ch in characters.iter() {
        if ch.char == '\t' {
            continue
        }
        info!("Pressed: '{}'", ch.char);
        if ch.char == CHAR_DELETE {
            if !selected.is_empty() {
                chars.drain(selected.range());
                index = selected.min;
                selected.stop();
                input.value = chars.iter().collect();
            } else if index > 0 {
                index -= 1;
                chars.remove(index);
                input.value = chars.iter().collect();
            }
        } else {
            if !selected.is_empty() {
                chars.drain(selected.range());
                index = selected.min;
                selected.stop();
            }
            chars.insert(index, ch.char);
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
    let mut selection_from = 0.;
    let mut selection_to = 0.;
    let mut text_width = 0.;
    let Some(font) = fonts.get(&text.sections[0].style.font) else { return };
    let font_size = text.sections[0].style.font_size;
    for (idx, ch) in chars.iter().enumerate() {
        let advance = get_char_advance(*ch, font, font_size);
        text_width += advance;
        if idx < index {
            position_from_start += advance;
        }
        if idx < selected.min {
            selection_from += advance;
        }
        if idx < selected.max {
            selection_to += advance;
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
    selection_from += offset;
    selection_to += offset;
    selection_to = selection_to.min(container_width);
    let cursor_position = position_from_start + offset;
    // let offset = (position_from_start - container_width).max(0.);
    if let Ok(mut cursor_style) = styles.get_mut(input.cursor) {
        cursor_style.position.left = Val::Px(cursor_position);
    }
    if let Ok(mut contaienr_style) = styles.get_mut(input.container) {
        contaienr_style.padding.left = Val::Px(offset);
    }
    if let Ok(mut selection_style) = styles.get_mut(input.selection) {
        if !selected.is_empty() {
            selection_style.display = Display::Flex;
            selection_style.position.left = Val::Px(selection_from);
            selection_style.size.width = Val::Px(selection_to - selection_from);
        } else {
            selection_style.display = Display::None;
        }
    }
    if input.index != index {
        input.index = index;
    }
    if input.selected != selected {
        input.selected = selected;
    }
    
    info!("{}:process_keyboard_input: Resulting index: {}, cursor: {}, selection: {:?}",  frame, input.index, cursor_position, selected);
    
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
    mut events: EventReader<PointerInput>,
    mut inputs: Query<(Entity, &mut TextInput, &mut Element)>,
    texts: Query<&Text>,
    styles: Query<(&Style, &GlobalTransform, &Node)>,
    fonts: Res<Assets<Font>>,
    keyboard: Res<Input<KeyCode>>,
    diag: Res<Diagnostics>,
) {
    for evt in events.iter().filter(|s| s.down() || s.dragging() || s.drag_stop()) {
        for (entity, mut input, mut element) in inputs.iter_mut() {
            if evt.drag_start() && evt.contains(entity) {
                let start = input.index;
                input.selected.start(start);
                continue;
            }
            if evt.down() && !evt.contains(entity) {
                continue;
            }
            if evt.dragging() && !evt.dragging_from(entity) {
                continue;
            }
            let Ok((container, tr, node)) = styles.get(input.container) else { continue };
            let mut offset = if let Val::Px(offset) = container.padding.left {
                offset
            } else {
                0.
            };
            let Ok(text) = texts.get(input.text) else { continue };
            let Some(font) = fonts.get(&text.sections[0].style.font) else { continue };
            let font_size = text.sections[0].style.font_size;
            let pos = (evt.pos - tr.translation().truncate() + node.size() * 0.5).x;
            let mut index = 0;
            let mut idx_found = false;
            let mut word_start = 0;
            let mut word_end = 0;
            let mut whitespace = false;
            for (idx, ch) in input.value.chars().enumerate() {
                let advance = get_char_advance(ch, font, font_size);
                if offset < pos && !idx_found {
                    index = idx;
                    if offset + advance * 0.5 < pos {
                        index += 1;
                    }
                } else {
                    idx_found = true;
                }
                offset += advance;
                if !whitespace && !ch.is_whitespace() {
                    word_end = idx + 1;
                }  else if whitespace && ch.is_whitespace() {
                    word_end = idx + 1;
                } else if idx_found {
                    break;
                } else {
                    whitespace = !whitespace;
                    word_start = idx;
                    word_end = idx + 1;
                }
            }

            let mut selected = input.selected.clone();
            let shift = keyboard.any_pressed([KeyCode::LShift, KeyCode::RShift]);
            info!("presses: {}", evt.presses());
            if evt.down() && evt.presses() == 2 {
                selected.start(word_start);
                selected.extend(word_end);
                index = selected.max;
            } else if evt.down() && evt.presses() > 2 {
                selected.start(0);
                selected.extend(input.value.chars().count());
                index = selected.max;
            } else if evt.dragging() || evt.down() && shift {
                selected.extend(index);
            } else if evt.down() {
                selected.stop();
            }
            if input.index != index {
                input.index = index;
                element.invalidate();
            }
            if input.selected != selected {
                input.selected = selected;
                element.invalidate();
            }
            let frame = diag.get(FrameTimeDiagnosticsPlugin::FRAME_COUNT).unwrap().average().unwrap_or_default();
            info!("{}:process_mouse: Clicked relative: {:.2}, idx={}, offset={}, focused: {}, range: {:?}", frame, pos, index, offset, element.focused(), selected);

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