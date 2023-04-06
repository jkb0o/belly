use belly_core::build::*;
use belly_macro::*;
use bevy::prelude::*;

#[doc(hidden)]
pub(crate) struct LayoutPlugin;

pub mod prelude {
  pub use super::RowWidgetExtension;
  pub use super::ColumnWidgetExtension;
  pub use super::CenterWidgetExtension;
  pub use super::WrapWidgetExtension;
}

impl Plugin for LayoutPlugin {
  fn build(&self, app: &mut App) {
      app.register_widget::<RowWidget>();
      app.register_widget::<ColumnWidget>();
      app.register_widget::<CenterWidget>();
      app.register_widget::<WrapWidget>();
  }
}

#[widget]
#[styles(
    row {
      flex-direction: row;
      justify-content: space-around;
      align-content: center;
    }
)]
fn row(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default())
        .insert(Interaction::None)
        .push_children(&content);
}

#[widget]
#[styles(
    column {
      flex-direction: column;
      justify-content: space-around;
      align-content: center;
    }
)]
fn column(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default())
        .insert(Interaction::None)
        .push_children(&content);
}

#[widget]
#[styles(
    center {
      justify-content: center;
      align-content: center;
    }
)]
fn center(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default())
        .insert(Interaction::None)
        .push_children(&content);
}

#[widget]
#[styles(
    wrap {
      justify-content: center;
      align-content: center;
    }
)]
fn wrap(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.insert(ElementBundle::default())
        .insert(Interaction::None)
        .push_children(&content);
}