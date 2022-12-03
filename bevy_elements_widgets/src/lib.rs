pub mod common;
pub mod input;
use bevy::prelude::Plugin;

#[derive(Default)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(input::InputPlugins);
        app.add_plugin(common::CommonsPlugin);
    }
}

pub mod prelude {
    pub use crate::common::CommonWidgetsExtension;
    pub use crate::common::Label;
    pub use crate::input::TextInput;
    pub use crate::input::TextInputWidgetExtension;
}
