use belly::prelude::*;
use bevy::{prelude::*, render::camera::ScalingMode, text::TextSettings, window::PrimaryWindow};
// TODO: rename to ui-scale, add example comment

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, scale)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 5.)),
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1024.,
                min_height: 768.,
            },
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("icon.png"),
        ..default()
    });
    commands.insert_resource(UiScale(1.));
    commands.insert_resource(TextSettings {
        allow_dynamic_font_size: true,
        ..default()
    });
    commands.add(eml! {
        <body s:flex-direction="column">
            <span s:height="15px"/>
            <span s:width="50px" s:height="15px" s:background-color="rebeccapurple"/>
            <span s:margin-left="50px" s:font-size="36px">"Resize the window to see the difference."</span>
            <span s:width="50px" s:height="15px" s:background-color="rebeccapurple"/>
        </body>
    })
}

pub fn scale(
    mut cached_size: Local<Vec2>,
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(primary) = windows.iter().next() else {
        return;
    };
    let ww = primary.width();
    let wh = primary.height();
    if cached_size.x == ww && cached_size.y == wh {
        return;
    }
    cached_size.x = ww;
    cached_size.y = wh;

    let scale_h = ww / 1024.0;
    let scale_w = wh / 768.0;
    ui_scale.0 = scale_h.min(scale_w) as f64;
}
