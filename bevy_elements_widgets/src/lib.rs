pub mod input;
pub mod text_line;
use bevy::prelude::Plugin;


#[derive(Default)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(input::InputPlugins);
        app.add_plugin(text_line::TextLinePlugin);
    }
}