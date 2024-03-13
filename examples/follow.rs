// examples/follow.rs
// cargo run --example follow
use belly::build::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, move_sprite)
        .add_systems(Update, spawn_sprites)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(eml! {
    <body s:padding="50px">
        "Press space to spawn sprite. "
        "The <follow> will bind own absolute position "
        "to target's GlobalTransform."
    </body> })
}

fn spawn_sprites(
    keys: Res<Input<KeyCode>>,
    mut elements: Elements,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let sprite = commands
            .spawn(SpriteBundle {
                texture: assets.load("icon.png"),
                transform: Transform {
                    translation: Vec3 {
                        x: -200.,
                        y: 200.,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            })
            .id();
        elements.select("body").add_child(eml! {
            <follow target=sprite>
                <span s:width="150px" s:height="50px">
                    <span s:background-color="#2d2d2d" s:width="100%" s:height="100%" s:justify-content="center" s:align-items="center">
                        "Hello world?"
                    </span>
                </span>
            </follow>
        });
    }
}

fn move_sprite(mut sprites: Query<&mut Transform, With<Sprite>>, time: Res<Time>) {
    let delta = time.delta_seconds();
    for mut sprite in sprites.iter_mut() {
        sprite.translation += Vec3::new(25. * delta, -25. * delta, 0.)
    }
}
