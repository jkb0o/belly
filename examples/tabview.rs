// examples/tabview.rs
// cargo run --example tabview
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component, Default)]
struct TabController;
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::parse(
        "
        .hidden {
            display: none;
        }
    ",
    ));
    commands.add(eml! {
      <body s:padding="20px">
        <buttongroup on:value_change=connect!(|ctx| {
            let ev = ctx.event();
            ctx.select(ev.old_value()).add_class("hidden");
            ctx.select(ev.new_value()).remove_class("hidden");
        })>
          <button value=".tab1" pressed>"Tab 1"</button>
          <button value=".tab2">"Tab 2"</button>
          <button value=".tab3">"Tab 3"</button>
        </buttongroup>
        <br/>
        <div c:content>
          <div c:tab1>"Tab 1 content"</div>
          <div c:tab2 c:hidden>"Tab 2 content"</div>
          <div c:tab3 c:hidden>"Tab 3 content"</div>
        </div>
      </body>
    });
}
