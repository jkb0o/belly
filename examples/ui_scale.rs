use bevy::{prelude::*, render::camera::ScalingMode, text::TextSettings};
use belly::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .add_system(scale)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Auto {
                min_width: 1024.,
                min_height: 768.,
            },
            ..default()
        },
        ..default()
    });
    commands.spawn(EmlScene::new(asset_server.load("test.eml")));
    commands.spawn(SpriteBundle {
        texture: asset_server.load("icon.png"),
        ..default()
    });
    commands.insert_resource(UiScale { scale: 1. });
    commands.insert_resource(TextSettings {
        allow_dynamic_font_size: true,
        ..default()
    });
}

pub fn scale(mut cached_size: Local<Vec2>, mut ui_scale: ResMut<UiScale>, windows: Res<Windows>) {
    let ww = windows.primary().width();
    let wh = windows.primary().height();
    if cached_size.x == ww && cached_size.y == wh {
        return;
    }
    cached_size.x = ww;
    cached_size.y = wh;

    let scale_h = ww / 1024.0;
    let scale_w = wh / 768.0;
    ui_scale.scale = scale_h.min(scale_w) as f64;
}
