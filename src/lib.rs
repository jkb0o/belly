pub use belly_core as core;

pub mod prelude {
    use belly_core::ElementsCorePlugin;
    use belly_widgets::WidgetsPlugin;
    use bevy::prelude::*;

    pub use belly_core::prelude::*;
    pub use belly_macro::eml;
    pub use belly_widgets::prelude::*;

    pub struct BellyPlugin;
    impl Plugin for BellyPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugin(ElementsCorePlugin);
            app.add_plugin(WidgetsPlugin);
        }
    }
}

pub mod build {
    pub use super::prelude::*;
    pub use belly_core::build::*;
    pub use belly_macro::widget;
}
