use std::any::Any;

use bevy::prelude::*;
use bevy_mod_props::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, log_transforms)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn(Style {
        grid_auto_rows: vec![GridTrack::px(23.)],
        ..default()
    });

    let entity = commands.spawn_empty().id();
    let a = move |entity: Entity| {
        info!("empty e: {entity:?}");
        23
    };
    let b: Box<dyn Fn(Entity) -> usize> = Box::new(a);
    // let b = Box::new(a);
    let x: Box<dyn Any> = Box::new(b);
    let y = x.downcast::<Box<dyn Fn(Entity) -> usize>>();
    if y.is_ok() {
        info!("im good downcaster");
    } else {
        info!("bait pidr");
    }
}

use bevy_mod_props::usage::*;

fn log_transforms(
    transforms: Query<&Transform>,
    colors: Query<&BackgroundColor>,
    styles: Query<&mut Style, Changed<Style>>,

) {
    for tr in transforms.iter() {
        tr.props().translation().x();
    }
    for bg in colors.iter() {
        bg.props().color().a();
    }
    for style in styles.iter() {
        let g = style.props().grid_auto_rows().at(0).get();
        info!("grid rows: {g:?}");
        let u = style.props().lookup("grid_auto_rows[0]").unwrap().get();
        info!("grid rows untyped: {u:?}");

        info!("aspect before change: {:?}", style.aspect_ratio);
        let untyped = style.props().lookup("aspect_ratio.value").unwrap().get();
        info!("aspect after change: {:?}", style.aspect_ratio);
        info!("untyped: {untyped:?}");
    }    
}

