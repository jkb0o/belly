use super::range::*;
use belly_core::build::*;
use belly_macro::*;
use bevy::prelude::*;

#[doc(hidden)]
pub(crate) struct CommonsPlugin;

pub mod prelude {
    pub use super::BodyWidgetExtension;
    pub use super::BrWidgetExtension;
    pub use super::BrlWidgetExtension;
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
        app.register_widget::<BrWidget>();
        app.register_widget::<BrlWidget>();
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
        width: 100%;
        height: 100%;
        align-content: flex-start;
        align-items: flex-start;
        flex-wrap: wrap;
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
#[styles(br { flex-basis: 100%; })]
/// The `<br/>` tag inserts single line break. `<br/>` height is
/// zero, so combining multiple `<br/>` tags has no effect. Use
/// [`<brl/>`](BrlWidget) if you want to insert extra empty line.
fn br(ctx: &mut WidgetContext) {
    ctx.insert(ElementBundle::default());
}

#[widget]
#[styles( brl {flex-basis: 100%; })]
/// The `<brl/>` tag inserts line break **and** extra empty line
/// with the height of the current font-size. If you only need
/// to insert single line break use [`<br/>`](br) tag instead.
fn brl(ctx: &mut WidgetContext) {
    ctx.insert(TextElementBundle::default());
}

#[widget]
#[styles(
    div {
        flex-wrap: wrap;
        flex-basis: 100%;
    }
)]
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
    ctx.commands()
        .add(from!(this, Label: value) >> to!(this, Text:sections[0].value));
    ctx.insert(TextElementBundle::default());
}
