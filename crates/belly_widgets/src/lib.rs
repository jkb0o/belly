pub mod common;
pub mod img;
pub mod input;
pub mod range;
use bevy::prelude::Plugin;

#[derive(Default)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(common::CommonsPlugin);
        app.add_plugin(range::RangePlugin);
        app.add_plugin(img::ImgPlugin);
        app.add_plugin(input::InputPlugins);
    }
}

pub mod prelude {
    pub use crate::common::prelude::*;
    pub use crate::img::prelude::*;
    pub use crate::input::prelude::*;
}

pub mod tags {
    pub use belly_core::tags::*;
}
