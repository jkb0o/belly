pub mod common;
pub mod img;
pub mod input;
use bevy::prelude::Plugin;

#[derive(Default)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(img::ImgPlugin);
        app.add_plugin(input::InputPlugins);
        app.add_plugin(common::CommonsPlugin);
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
