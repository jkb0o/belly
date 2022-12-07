pub mod button;
pub mod text;

use bevy::prelude::Plugin;
pub use button::*;
pub use text::TextInput;
use text::TextInputPlugin;
pub use text::TextInputWidgetExtension;

pub struct InputPlugins;
impl Plugin for InputPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(TextInputPlugin);
        app.add_plugin(button::ButtonPlugin);
    }
}
