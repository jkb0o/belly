//! The `belly` is a plugin for a [Bevy](https://bevyengine.org/) engine that
//! helps to declaratively define a user interface with `eml` markup (macros & asset),
//! style it with a very CSS-like `ess` syntax, and define data flow using `from!` &
//! `to!` bind macros and/or connect to signals (events) with `connect!` macro.
//!
//! The project is at the early stage of development, pretty much experimental,
//! API is unstable and will change in future.
//!
//! The project documentations grows slowly, the best source of knowlage at
//! the moment is the [README](https://github.com/jkb0o/belly#about) and
//! [examples](https://github.com/jkb0o/belly/tree/main/examples).
//!
//! ### Example
//!
//! ```rust
//! // examples/color-picker.rs
//! // cargo run --example color-picker
//! use belly::prelude::*;
//! use bevy::prelude::*;
//! fn main() {
//!     # return;
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BellyPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! const COLORS: &[&'static str] = &[
//!     // from https://colorswall.com/palette/105557
//!     // Red     Pink       Purple     Deep Purple
//!     "#f44336", "#e81e63", "#9c27b0", "#673ab7",
//!     // Indigo  Blue       Light Blue Cyan
//!     "#3f51b5", "#2196f3", "#03a9f4", "#00bcd4",
//!     // Teal    Green      Light      Green Lime
//!     "#009688", "#4caf50", "#8bc34a", "#cddc39",
//!     // Yellow  Amber      Orange     Deep Orange
//!     "#ffeb3b", "#ffc107", "#ff9800", "#ff5722",
//! ];
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera2dBundle::default());
//!     commands.add(StyleSheet::load("color-picker.ess"));
//!     let colorbox = commands.spawn_empty().id();
//!     commands.add(eml! {
//!         <body>
//!             <span c:controls>
//!                 <slider c:red
//!                     bind:value=to!(colorbox, BackgroundColor:0|r)
//!                     bind:value=from!(colorbox, BackgroundColor:0.r())
//!                 />
//!                 <slider c:green
//!                     bind:value=to!(colorbox, BackgroundColor:0|g)
//!                     bind:value=from!(colorbox, BackgroundColor:0.g())
//!                 />
//!                 <slider c:blue
//!                     bind:value=to!(colorbox, BackgroundColor:0|b)
//!                     bind:value=from!(colorbox, BackgroundColor:0.b())
//!                 />
//!                 <slider c:alpha
//!                     bind:value=to!(colorbox, BackgroundColor:0|a)
//!                     bind:value=from!(colorbox, BackgroundColor:0.a())
//!                 />
//!             </span>
//!             <img c:colorbox-holder src="trbg.png">
//!                 <span {colorbox} c:colorbox
//!                     on:ready=run!(for colorbox |c: &mut BackgroundColor| c.0 = Color::WHITE)
//!                 />
//!             </img>
//!             <span c:colors>
//!             <for color in = COLORS>
//!                 <button on:press=run!(for colorbox
//!                     |c: &mut BackgroundColor| c.0 = Color::from_hex(color)
//!                 )>
//!                     <span s:background-color=*color s:width="100%" s:height="100%"/>
//!                 </button>
//!             </for>
//!             </span>
//!         </body>
//!     });
//! }
//! ```
//! ![Example][color_picker]
//!
//! ### Crate
//! The `belly` crate is just container crate that makes it easier to consume subcrates.
//! It has to main mods: `prelude` for using plugin and `build` for extending plugin.
//!
#![doc = ::embed_doc_image::embed_image!("color_picker", "docs/img/examples/color-picker.gif")]

pub use belly_core as core;
pub use belly_widgets as widgets;

/// `use belly::prelude::*` for adding the UI to your project
pub mod prelude {
    use belly_core::ElementsCorePlugin;
    use belly_widgets::WidgetsPlugin;
    use bevy::prelude::*;

    pub use belly_core::prelude::*;
    pub use belly_macro::eml;
    pub use belly_macro::ess;
    pub use belly_macro::run;
    pub use belly_widgets::prelude::*;

    pub struct BellyPlugin;
    impl Plugin for BellyPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(ElementsCorePlugin);
            app.add_plugins(WidgetsPlugin);
        }
    }
}

/// `use belly::build::*` for extending the `belly` plugin with custom elements & styles
pub mod build {
    pub use super::prelude::*;
    pub use belly_core::build::*;
    pub use belly_macro::widget;
}
