use crate::common::*;
use bevy::prelude::*;
use bevy_elements_core::*;
use bevy_elements_macro::*;

pub(crate) struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<button>();
    }
}

#[widget]
#[style("justify-content: space-around")]
#[style("align-content: center")]
#[style("min-width: 40px")]
#[style("min-height: 40px")]
#[signal(press, PointerInput, pressed)]
/// The `<button>` tag defines a clickable button.
/// Inside a `<button>` element you can put text (and tags
/// like `<i>`, `<b>`, `<strong>`, `<br>`, `<img>`, etc.)
fn button(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.render(eml! {
        <div c:button interactable>{content}</div>
    })
}
