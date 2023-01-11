pub mod common;
pub mod img;
pub mod input;
pub mod range;
use bevy::prelude::Plugin;

#[derive(Default)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(img::ImgPlugin);
        app.add_plugin(input::InputPlugins);
        app.add_plugin(common::CommonsPlugin);
        app.add_plugin(range::RangePlugin);
    }
}

pub mod prelude {
    #[doc(inline)]
    pub use crate::common::*;
    #[doc(inline)]
    pub use crate::img::*;
    #[doc(inline)]
    pub use crate::input::*;
}

pub mod tags {
    pub use belly_core::tags::*;
}
