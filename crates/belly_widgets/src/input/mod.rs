pub mod button;
pub mod slider;
pub mod text;

use bevy::prelude::Plugin;

pub mod prelude {
    pub use super::button::*;
    pub use super::slider::*;
    pub use super::text::TextInput;
    pub use super::text::TextInputWidgetExtension;
}

pub struct InputPlugins;
impl Plugin for InputPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(text::TextInputPlugin);
        app.add_plugin(button::ButtonPlugin);
        app.add_plugin(slider::SliderPlugin);
    }
}
