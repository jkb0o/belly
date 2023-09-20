use super::range::*;
use belly_core::build::*;
use belly_macro::*;
use bevy::prelude::*;

#[doc(hidden)]
pub(crate) struct CommonsPlugin;

pub mod prelude {
    pub use super::BodyWidgetExtension;
    pub use super::DivWidgetExtension;
    pub use super::LabelWidgetExtension;
    pub use super::ProgressbarWidgetExtension;
    pub use super::SpanWidgetExtension;
    pub use super::StrongWidgetExtension;

    pub use super::Label;
}

impl Plugin for CommonsPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<BodyWidget>();
        app.register_widget::<DivWidget>();
        app.register_widget::<LabelWidget>();
        // app.register_widget::<Label>();
        app.register_widget::<ProgressbarWidget>();
        app.register_widget::<SpanWidget>();
        app.register_widget::<StrongWidget>();
    }
}

#[widget]
#[styles(
    body {
        width: 100%;
        height: 100%;
    }
)]
/// The `<body>` tag defines a ui content (text, images, links, inputs, etc.).
/// It occupies the entire space of the window and should be treated as root
/// container for other elements.
///
/// > **_NOTE:_** in the future releases it is possible the only single `<body>`
/// > element on the page would be allowed.
///
/// <!-- widget-category: common -->
fn body(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default())
        .insert(Interaction::None)
        .push_children(&content);
}

#[widget]
/// The `<div>` tag is an empty container that is used to define
/// a division or a section. It does not affect the content or layout
/// and is used to group `eml` elements to be styled with `ess`.
fn div(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}

#[widget]
#[extends(RangeWidget)]
#[styles(
    progressbar {
        min-width: 26px;
        min-height: 26px;
    }
)]
fn progressbar(ctx: &mut WidgetContext) {
    let params = ctx.params();
    ctx.render(eml! {
        <range c:progress-bar params=params/>
    })
}

#[widget]
fn span(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}

#[widget]
#[styles( strong: { font: bold; })]
/// The `<strong>` tag highlights an important part of a text. It can be used
/// for such important contents, as warnings. This can be one sentence that gives
/// importance to the whole page, or it may be needed if you want to highlight
/// some words that are of greater importance compared to the rest of the content.
fn strong(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}

#[derive(Component, Default)]
pub struct Label {
    pub value: String,
}

#[widget]
#[param(value:String => Label:value)]
/// The `<label>` tag is a binable single line of text. It consumes
/// the children and renders the content of bindable `value` param.
fn label(ctx: &mut WidgetContext) {
    let this = ctx.this().id();
    ctx.add(from!(this, Label: value) >> to!(this, Text:sections[0].value));
    ctx.insert(TextElementBundle::default());
}
