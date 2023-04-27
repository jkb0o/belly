use bevy::prelude::*;
use bevy_stylebox::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(StyleboxPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let flat = Stylebox {
        slice: UiRect::all(Val::Percent(25.)),
        width: UiRect::all(Val::Px(8.)),
        texture: asset_server.load("rounded.basis"),
        modulate: Color::DARK_GRAY,
        ..default()
    };
    let tex = Stylebox {
        slice: UiRect::all(Val::Px(16.)),
        texture: asset_server.load("panel-blue.png"),
        ..default()
    };
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for stylebox in [flat, tex] {
                parent.spawn(StyleboxBundle {
                    stylebox,
                    style: Style {
                        size: Size::new(Val::Percent(40.), Val::Percent(80.)),
                        ..default()
                    },
                    ..default()
                })
                .insert(Cover::new(UiRect::all(Val::Px(2.)), Color::GRAY))
                .insert(Shadow::new(
                    UiRect::all(Val::Px(4.)),
                    Color::rgba(0., 0., 0., 0.25),
                ));
                break;
            }
        });
}
