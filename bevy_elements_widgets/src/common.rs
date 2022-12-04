use bevy::prelude::*;
use bevy_elements_core::*;
use bevy_elements_macro::*;

pub(crate) struct CommonsPlugin;

impl Plugin for CommonsPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<body>();
        app.register_widget::<div>();
        app.register_widget::<Label>();
        app.register_widget::<strong>();
    }
}

#[widget(
    "width: 100%",
    "height: 100%",
    "align-content: flex-start",
    "align-items: flex-start"
)]
/// The `<body>` tag defines a ui content (text, images, links, inputs, etc.).
/// It occupies the entire space of the window and should be treated as root
/// container for other elements.
///
/// > **_NOTE:_** in the future releases it is possible the only single `<body>`
/// > element on the page would be allowed.
///
/// <!-- widget-category: common -->
fn body(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default())
        .insert(Interaction::None)
        .push_children(&content);
}

#[widget]
/// The `<div>` tag is an empty container that is used to define
/// a division or a section. It does not affect the content or layout
/// and is used to group `eml` elements to be styled with `ess`.
fn div(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}

#[derive(Component, Widget)]
#[alias(label)]
/// The `<label>` tag is a binable single line line of text. It consumes
/// the children and renders the content of bindable `value` param:
/// ```rust
/// let input = commands.spawn_empty().id();
/// commands.add(eml! {
///     // just a single line of text
///     <label value="Hello world!"/>
///
///     // bind textinput.value to label.value
///     <textinput {input}/>
///     <label value=bind!(<= input, TextInput.value)/>
/// });
/// ```
pub struct Label {
    #[param( => this, Text.sections[0].value)]
    pub value: String,
}

impl WidgetBuilder for Label {
    fn setup(&mut self, ctx: &mut ElementContext) {
        ctx.insert(TextElementBundle::default());
    }
}

#[widget("font: default-bold")]
/// The `<strong>` tag highlights an important part of a text. It can be used
/// for such important contents, as warnings. This can be one sentence that gives
/// importance to the whole page, or it may be needed if you want to highlight
/// some words that are of greater importance compared to the rest of the content.
fn strong(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}
