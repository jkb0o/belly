use belly::build::*;
use belly_core::eml::Eml;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .register_type::<DataFor>()
        .register_type::<ViewFor>()
        // Defaults
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BellyPlugin)
        // Init
        .add_systems(Startup, setup)
        .add_systems(Update, render_collection_item)
        // .add_event::<DataEvent>()
        .run();
}

pub type RenderFunction = Box<dyn Fn(Entity) -> Eml>;

#[widget]
fn list(ctx: &mut WidgetContext) {
    let collection = ctx
        .param("collection".into())
        .expect("`collection` param is required for `<list/>`")
        .take::<Entity>()
        .expect("`collection` param is required for `<list/>`");

    // link data
    let link_to_view = DataFor(ctx.entity());

    let commands = ctx.commands();
    let mut data_entity_commands = commands.entity(collection);
    data_entity_commands.insert(link_to_view);

    // render container
    let components = (ViewFor(collection), Collection);
    ctx.render(eml! { <span with=components/> })
}

#[derive(Reflect, Component, PartialEq, Debug)]
pub struct Data;

#[derive(Reflect, Component, PartialEq, Debug)]
pub struct ViewFor(pub Entity);

#[derive(Reflect, Component, PartialEq, Debug)]
pub struct DataFor(pub Entity);

#[derive(Reflect, Component, PartialEq, Debug)]
pub struct Collection;

#[derive(Reflect, Component, PartialEq, Debug)]
pub struct InCollection(pub Entity);

#[derive(Reflect, Component, Deref, DerefMut, PartialEq, Debug)]
pub struct Item(pub String);

fn setup<'a>(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Data hierarchy
    let collection = commands
        .spawn((
            Data, Collection,
            // Name::new("Data Collection")
        ))
        .id();

    commands.entity(collection).with_children(|children| {
        children.spawn((Data, InCollection(collection), Item("1".into())));
        children.spawn((Data, InCollection(collection), Item("2".into())));
    });

    // Create view hierarchy and bind to data
    commands.add(eml! {
        <body>
          <label value="List:"/>
          <list collection=collection></list>
        </body>
    });
}

fn render_collection_item(
    mut commands: Commands,
    data_items: Query<(&Item, &InCollection), (With<Data>, Added<Item>)>,
    collection_views: Query<&DataFor, With<Collection>>,
    // should be data views it's like binding
) {
    for (item, in_collection) in data_items.iter() {
        // Get all views for that collection, should be iteration
        let Ok(data_for) = collection_views.get(in_collection.0) else {
            continue;
        };

        let view = data_for.0;

        // Add rendering
        let rendered_something = TextBundle::from_section(item.0.clone(), TextStyle::default());

        commands.entity(view).with_children(|children| {
            children.spawn(rendered_something);
        });
    }
}
