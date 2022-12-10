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
#[style(
    ".button",
    "align-content: center",
    "min-width: 40px",
    "min-height: 40px",
    "margin: 5px"
)]
#[style("button:hover .button-foreground", "background-color: white")]
#[style(
    ".button-shadow",
    "background-color: #4f4f4fb8",
    "top: 2px",
    "left: 2px",
    "bottom: -2px",
    "right: -2px"
)]
#[style(
    ".button-background",
    "margin: -2px",
    "padding: 1px",
    "background-color: #2f2f2f",
    "left: 0px",
    "right: 0px",
    "width: 100%",
    "height: 100%"
)]
#[style(
    ".button-foreground",
    "width: 100%",
    "height: 100%",
    "background-color: #dfdfdf",
    "color: #2f2f2f",
    "justify-content: center",
    "align-content: center",
    "align-items: center"
)]
#[style(".button-foreground *", "color: #2f2f2f")]
#[signal(press, PointerInput, pressed)]
/// The `<button>` tag defines a clickable button.
/// Inside a `<button>` element you can put text (and tags
/// like `<i>`, `<b>`, `<strong>`, `<br>`, `<img>`, etc.)
fn button(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.render(eml! {
        <span c:button interactable>
            <span c:button-shadow s:position-type="absolute"/>
            <span c:button-background>
                <span c:button-foreground>
                    {content}
                </span>
            </span>
        </span>
    })
}
