use crate::common::*;
use ab_glyph::ScaleFont;
use belly_core::{build::*, input};
use belly_macro::*;
use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::common::Label;

pub mod prelude {
    pub use super::TextInput;
    pub use super::TextinputWidgetExtension;
}

const CURSOR_WIDTH: f32 = 2.;

pub struct TextInputPlugin;
impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<TextinputWidget>();
        app.add_systems(Update, blink_cursor);
        app.add_systems(
            PreUpdate,
            (process_cursor_focus, process_mouse, process_keyboard_input)
                .chain()
                .in_set(input::InputSystemsSet),
        );
    }
}

#[widget]
#[param(value: String => TextInput:value)]
#[styles = TEXTINPUT_STYLES]
fn textinput(ctx: &mut WidgetContext, ti: &mut TextInput) {
    let this = ctx.this().id();
    let cursor = ti.cursor;
    let text = ti.text;
    let container = ti.container;
    let selection = ti.selection;
    // let a = belly_core::relations::bind::ToComponentWithoutTransformer {
    //     id: belly_core::relations::bind::bind_id::<Label>("value"),
    //     target: text,
    //     reader: |c: &::bevy::prelude::Mut<Label>| &c.value,
    //     writer: |c: &mut ::bevy::prelude::Mut<Label>| &mut c.value,
    // };
    ctx.add(from!(this, TextInput: value) >> to!(text, Label: value));
    ctx.render(eml! {
        <span interactable="block" c:text-input c:text-input-border>
            <span c:text-input-background>
                <span {container} c:text-input-container>
                    <span {selection}
                        c:text-input-selection
                        s:display=managed()
                        s:left=managed()
                        s:width=managed()
                    />
                    <label {text} c:text-input-value/>
                    <span {cursor} c:text-input-cursor
                        s:position-type="absolute"
                        s:width=format!("{:.0}px", CURSOR_WIDTH)
                        s:display=managed()
                    />
                </span>
            </span>
        </span>
    });
}

ess_define! {
    TEXTINPUT_STYLES,
    .text-input {
        width: 200px;
    }
    .text-input-border {
        background-color: #2f2f2f00;
        padding: 1px;
    }
    .text-input-background {
        padding: 1px;
        width: 100%;
        height: 100%;
        background-color: #efefef;
    }
    .text-input-container {
        width: 100%;
        height: 100%;
        width: 100%;
        overflow: clip;
    }
    .text-input-selection {
        position-type: absolute;
        height: 100%;
        background-color: #9f9f9f;
    }
    .text-input-value {
        color: #2f2f2f;
    }
    .text-input-cursor {
        top: 1px;
        bottom: 1px;
        background-color: #2f2f2f;
    }
}

#[derive(Component)]
// #[alias(textinput)]
/// The `<inputtext>` tag specifies a text input field
/// where the user can enter data.
pub struct TextInput {
    pub value: String,
    index: usize,
    selected: Selection,
    text: Entity,
    container: Entity,
    selection: Entity,
    cursor: Entity,
}

impl FromWorldAndParams for TextInput {
    fn from_world_and_params(world: &mut World, params: &mut belly_core::eml::Params) -> Self {
        TextInput {
            value: params.try_get("value").unwrap_or_default(),
            index: 0,
            selected: Selection::default(),
            text: world.spawn_empty().id(),
            container: world.spawn_empty().id(),
            selection: world.spawn_empty().id(),
            cursor: world.spawn_empty().id(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Selection {
    min: usize,
    max: usize,
    started: Option<usize>,
}

impl Selection {
    pub fn new() -> Selection {
        Selection {
            min: 0,
            max: 0,
            started: None,
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
        } else if value < self.min {
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

    pub fn range(&self) -> std::ops::Range<usize> {
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
    state: f32,
}

fn get_char_advance(ch: char, font: &Font, font_size: f32) -> f32 {
    let font = ab_glyph::Font::as_scaled(&font.font, font_size);
    let glyph = font.glyph_id(ch);
    font.h_advance(glyph)
}

fn process_keyboard_input(
    changed_elements: Query<(), Changed<Element>>,
    mut keyboard_input: EventReader<KeyboardInput>,
    keyboard: Res<Input<KeyCode>>,
    fonts: Res<Assets<Font>>,
    nodes: Query<&Node>,
    mut characters: EventReader<ReceivedCharacter>,
    mut inputs: Query<(Entity, &mut TextInput, &Element)>,
    mut cursors: Query<&mut TextInputCursor>,
    mut styles: Query<&mut Style>,
    texts: Query<&Text>,
) {
    let Some((entity, mut input)) = inputs
        .iter_mut()
        .filter(|(_, _, e)| e.focused())
        .map(|(e, i, _)| (e, i))
        .next()
    else {
        return;
    };
    if characters.is_empty() && keyboard_input.is_empty() && !changed_elements.contains(entity) {
        return;
    }

    let Ok(text) = texts.get(input.text) else {
        return;
    };

    // not shure how it behaves on Windows or *nix,
    // may be platform dependent compilation here?
    let cmd = keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let mut index = input.index;
    let mut selected = input.selected.clone();

    let mut chars: Vec<_> = input.value.chars().collect();
    for ch in keyboard_input.read() {
        if !ch.state.is_pressed() {
            continue;
        }
        let Some(code) = ch.key_code else { continue };
        match code {
            KeyCode::Left if !cmd => {
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
            }
            KeyCode::Right if !cmd => {
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
            }

            KeyCode::Up | KeyCode::Home | KeyCode::Left => {
                if !shift {
                    selected.stop();
                }
                let prev_index = index;
                index = 0;
                if shift {
                    selected.extend(prev_index);
                    selected.extend(index);
                }
            }
            KeyCode::Down | KeyCode::End | KeyCode::Right => {
                if !shift {
                    selected.stop();
                }
                let prev_index = index;
                index = chars.len();
                if shift {
                    selected.extend(prev_index);
                    selected.extend(index);
                }
            }
            KeyCode::Back => {
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
            }
            KeyCode::Delete => {
                if !selected.is_empty() {
                    chars.drain(selected.range());
                    index = selected.min;
                    selected.stop();
                    input.value = chars.iter().collect();
                } else {
                    if chars.len() > index {
                        chars.remove(index);
                        input.value = chars.iter().collect();
                    }
                }
            }
            _ => (),
        }
    }
    for ch in characters
        .read()
        .map(|c| c.char)
        .filter(|c| !c.is_control())
    {
        if !selected.is_empty() {
            chars.drain(selected.range());
            index = selected.min;
            selected.stop();
        }
        chars.insert(index, ch);
        input.value = chars.iter().collect();
        index += 1;
    }

    if let Ok(mut cursor) = cursors.get_mut(input.cursor) {
        cursor.state = 1.;
    }
    let Ok(node) = nodes.get(input.container) else {
        return;
    };
    let container_width = node.size().x;
    let mut position_from_start = 0.;
    let mut selection_from = 0.;
    let mut selection_to = 0.;
    let mut text_width = 0.;
    let Some(font) = fonts.get(&text.sections[0].style.font) else {
        return;
    };
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
            _ => 0.,
        }
    } else {
        0.
    };
    if offset + position_from_start < 0. {
        offset = -position_from_start;
    }
    if offset + position_from_start > container_width - CURSOR_WIDTH {
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
        cursor_style.left = Val::Px(cursor_position);
    }
    if let Ok(mut contaienr_style) = styles.get_mut(input.container) {
        contaienr_style.padding.left = Val::Px(offset);
    }
    if let Ok(mut selection_style) = styles.get_mut(input.selection) {
        if !selected.is_empty() {
            selection_style.display = Display::Flex;
            selection_style.left = Val::Px(selection_from);
            selection_style.width = Val::Px(selection_to - selection_from);
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
}

fn process_cursor_focus(
    mut commands: Commands,
    mut input: Query<(&mut TextInput, &Element), Changed<Element>>,
    cursors: Query<&TextInputCursor>,
    mut styles: Query<&mut Style>,
) {
    for (mut input, element) in input.iter_mut() {
        if element.focused() && !cursors.contains(input.cursor) {
            commands
                .entity(input.cursor)
                .insert(TextInputCursor::default());
        }
        if !element.focused() && !cursors.contains(input.cursor) {
            if let Ok(mut style) = styles.get_mut(input.cursor) {
                style.display = Display::None;
            }
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
) {
    for evt in events
        .read()
        .filter(|s| s.down() || s.dragging() || s.drag_stop())
    {
        for (entity, mut input, mut element) in inputs.iter_mut() {
            if evt.drag_start() && evt.contains(entity) {
                let start = input.index;
                input.selected.start(start);
                continue;
            }
            if evt.down() && !evt.contains(entity) {
                continue;
            }
            if evt.dragging() && !evt.is_dragging_from(entity) {
                continue;
            }
            let Ok((container, tr, node)) = styles.get(input.container) else {
                continue;
            };
            let mut offset = if let Val::Px(offset) = container.padding.left {
                offset
            } else {
                0.
            };
            let Ok(text) = texts.get(input.text) else {
                continue;
            };
            let Some(font) = fonts.get(&text.sections[0].style.font) else {
                continue;
            };
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
                } else if whitespace && ch.is_whitespace() {
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
            let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            if evt.down() && evt.presses() == 2 {
                selected.start(word_start);
                selected.extend(word_end);
                index = selected.max;
            } else if evt.down() && evt.presses() > 2 {
                selected.start(0);
                selected.extend(input.value.chars().count());
                index = selected.max;
            } else if evt.dragging() || evt.drag_stop() || evt.down() && shift {
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
        }
    }
}

fn blink_cursor(time: Res<Time>, mut cursor: Query<(&mut TextInputCursor, &mut Style)>) {
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
