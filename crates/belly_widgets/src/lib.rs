pub mod common;
pub mod follow;
pub mod img;
pub mod input;
pub mod range;
use bevy::prelude::Plugin;

#[derive(Default)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(common::CommonsPlugin);
        app.add_plugins(range::RangePlugin);
        app.add_plugins(img::ImgPlugin);
        app.add_plugins(input::InputPlugins);
        app.add_plugins(follow::FollowPlugin);
    }
}

pub mod prelude {
    pub use crate::common::prelude::*;
    pub use crate::follow::prelude::*;
    pub use crate::img::prelude::*;
    pub use crate::input::prelude::*;
}

pub mod tags {
    pub use belly_core::tags::*;
}
