use bevy::prelude::*;
use bevy_elements_core::*;
use bevy_elements_macro::*;

#[doc(hidden)]
pub(crate) struct CommonsPlugin;

impl Plugin for CommonsPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<body>();
        app.register_widget::<br>();
        app.register_widget::<brl>();
        app.register_widget::<div>();
        app.register_widget::<Label>();
        app.register_widget::<strong>();
    }
}

#[widget]
#[style("width: 100%")]
#[style("width: 100%")]
#[style("height: 100%")]
#[style("align-content: flex-start")]
#[style("align-items: flex-start")]
#[style("flex-wrap: wrap")]
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
#[style("flex-basis: 100%")]
/// The `<br/>` tag inserts single line break. `<br/>` height is
/// zero, so combining multiple `<br/>` tags has no effect. Use
/// [`<brl/>`](brl) if you want to insert extra empty line.
fn br(ctx: &mut ElementContext) {
    ctx.insert(ElementBundle::default());
}

#[widget]
#[style("flex-basis: 100%")]
/// The `<brl/>` tag inserts line break **and** extra empty line
/// with the height of the current font-size. If you only need
/// to insert single line break use [`<br/>`](br) tag instead.
fn brl(ctx: &mut ElementContext) {
    ctx.insert(TextElementBundle::default());
}

#[widget]
#[style("flex-wrap: wrap")]
#[style("flex-basis: 100%")]
/// The `<div>` tag is an empty container that is used to define
/// a division or a section. It does not affect the content or layout
/// and is used to group `eml` elements to be styled with `ess`.
fn div(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}

#[derive(Component, Widget)]
#[alias(label)]
/// The `<label>` tag is a binable single line of text. It consumes
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

    // fn connect<C:Component>(
    //     world: &mut World,
    //     signal: &'static str,
    //     source: Entity,
    //     target: ConnectionTo<C>
    // ) {
    //     match signal {
    //         "press" => {
    //             target
    //                 .filter(|e: &PointerInput| e.pressed())
    //                 .from(source)
    //                 .write(world);
    //         },
    //         _ => error!("Don't know how to connect '{signal}' signal")
    //     };
    // }
}

#[widget]
fn span(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}

#[widget]
#[style("font: bold")]
/// The `<strong>` tag highlights an important part of a text. It can be used
/// for such important contents, as warnings. This can be one sentence that gives
/// importance to the whole page, or it may be needed if you want to highlight
/// some words that are of greater importance compared to the rest of the content.
fn strong(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default()).push_children(&content);
}
