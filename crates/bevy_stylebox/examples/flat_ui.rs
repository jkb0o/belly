use bevy::prelude::*;
use bevy_stylebox::*;

// You can play with outer/inner radius to
// change the window border
const INNER_RADIUS: f32 = 12.;
const OUTER_RADIUS: f32 = 16.;
const BUTTON_RADIUS: f32 = 6.;
const MESSAGE: &str = "
  This example demonstrate how
to use circle texture to create
    UI with rounded corners.
Resize application window to see
     how the content feels.
";

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(StyleboxPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    #[cfg(feature = "basis-universal")]
    let circle = asset_server.load("circle.basis");
    #[cfg(not(feature = "basis-universal"))]
    let circle = asset_server.load("circle.png");
    let box_round_all_outer = Stylebox {
        slice: UiRect::all(Val::Percent(50.)),
        width: UiRect::all(Val::Px(OUTER_RADIUS)),
        texture: circle.clone(),
        modulate: Color::DARK_GRAY.into(),
        ..default()
    };
    let box_round_bot_inner = Stylebox {
        slice: UiRect::all(Val::Percent(50.)),
        width: UiRect::new(
            Val::Px(INNER_RADIUS),
            Val::Px(INNER_RADIUS),
            Val::Px(0.),
            Val::Px(INNER_RADIUS),
        ),
        texture: circle.clone(),
        modulate: Color::WHITE,
        ..default()
    };
    let box_round_all_button = Stylebox {
        slice: UiRect::all(Val::Percent(50.)),
        width: UiRect::all(Val::Px(BUTTON_RADIUS)),
        texture: circle.clone(),
        modulate: Color::DARK_GRAY,
        ..default()
    };
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                padding: UiRect::all(Val::Px(200.)),
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // WINDOW

            parent
                .spawn(StyleboxBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        min_size: Size::new(Val::Undefined, Val::Px(250.)),
                        // align_self: AlignSelf::FlexEnd,
                        // align_content: AlignContent::Stretch,
                        // align_items: AlignItems::FlexStart,
                        size: Size::new(Val::Px(500.), Val::Auto),
                        ..default()
                    },

                    stylebox: box_round_all_outer.clone(),
                    ..default()
                })
                .with_children(|parent| {
                    // HEADER

                    parent
                        .spawn(NodeBundle {
                            background_color: Color::NONE.into(),
                            style: Style {
                                justify_content: JustifyContent::SpaceBetween,
                                align_self: AlignSelf::Stretch,
                                size: Size::new(Val::Auto, Val::Px(32.)),
                                padding: UiRect::new(
                                    Val::Px(8.),
                                    Val::Px(10.),
                                    Val::Px(10.),
                                    Val::Auto,
                                ),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // HEADER TEXT

                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "Window Header".to_string(),
                                    TextStyle {
                                        font: asset_server.load("SourceCodePro-ExtraLight.ttf"),
                                        font_size: 20.,
                                        color: Color::WHITE,
                                    },
                                ),
                                style: Style {
                                    size: Size::new(Val::Undefined, Val::Undefined),
                                    max_size: Size::new(Val::Undefined, Val::Px(20.)),
                                    ..default()
                                },
                                ..default()
                            });

                            // HEADER BUTTON

                            parent
                                .spawn(ImageBundle {
                                    image: UiImage {
                                        texture: circle.clone(),
                                        ..default()
                                    },
                                    style: Style {
                                        padding: UiRect::all(Val::Px(2.)),
                                        size: Size::new(Val::Px(20.), Val::Px(20.)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(ImageBundle {
                                        background_color: Color::DARK_GRAY.into(),
                                        image: UiImage {
                                            texture: asset_server.load("cross.png"),
                                            ..default()
                                        },
                                        style: Style {
                                            size: Size::new(Val::Px(16.), Val::Px(16.)),
                                            ..default()
                                        },
                                        ..default()
                                    });
                                });
                        });

                    // CONTENT

                    let bw = OUTER_RADIUS - INNER_RADIUS;
                    parent
                        .spawn(StyleboxBundle {
                            style: Style {
                                flex_grow: 1.,
                                margin: UiRect::new(
                                    Val::Px(bw),
                                    Val::Px(bw),
                                    Val::Px(8.),
                                    Val::Px(bw),
                                ),
                                justify_content: JustifyContent::SpaceAround,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(8.)),
                                ..default()
                            },
                            stylebox: box_round_bot_inner.clone(),
                            ..default()
                        })
                        .with_children(|parent| {
                            // CONTENT TEXT

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_grow: 1.,
                                        justify_content: JustifyContent::Center,
                                        // align_content: AlignContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            MESSAGE.to_string(),
                                            TextStyle {
                                                font: asset_server
                                                    .load("SourceCodePro-ExtraLight.ttf"),
                                                font_size: 20.,
                                                color: Color::BLACK,
                                            },
                                        ),
                                        style: Style { ..default() },
                                        ..default()
                                    });
                                });

                            // OK BUTTON

                            parent
                                .spawn(StyleboxBundle {
                                    stylebox: box_round_all_button,
                                    style: Style {
                                        size: Size::new(Val::Px(100.), Val::Px(32.)),
                                        justify_content: JustifyContent::Center,
                                        // align_content: AlignContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            "OK".to_string(),
                                            TextStyle {
                                                font: asset_server
                                                    .load("SourceCodePro-ExtraLight.ttf"),
                                                font_size: 20.,
                                                color: Color::WHITE,
                                            },
                                        ),
                                        style: Style { ..default() },
                                        ..default()
                                    });
                                });
                        });
                });
        });
}
