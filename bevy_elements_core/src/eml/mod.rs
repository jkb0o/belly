use std::sync::Arc;

use crate::{
    attributes::Attribute, Element, ElementBuilderRegistry, PropertyExtractor, PropertyValidator,
};
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use tagstr::*;

use self::build::ElementContextData;
pub mod build;
pub mod content;
mod parse;

#[derive(Default)]
pub struct EmlPlugin;

impl Plugin for EmlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<EmlAsset>();
        let extractor = app
            .world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .clone();
        let validator = app
            .world
            .get_resource_or_insert_with(PropertyValidator::default)
            .clone();
        let registry = app
            .world
            .get_resource_or_insert_with(ElementBuilderRegistry::default)
            .clone();
        app.add_asset_loader(EmlLoader {
            validator,
            extractor,
            registry,
        });
        app.add_system(update_eml_scene);
    }
}

pub enum EmlNode {
    Element(EmlElement),
    Text(String),
}

#[derive(Default)]
pub struct EmlElement {
    name: Tag,
    attributes: HashMap<String, String>,
    children: Vec<EmlNode>,
}

impl EmlElement {
    pub fn new(name: Tag) -> EmlElement {
        EmlElement { name, ..default() }
    }
}

#[derive(Component)]
pub struct EmlScene {
    asset: Handle<EmlAsset>,
}

impl EmlScene {
    pub fn new(asset: Handle<EmlAsset>) -> EmlScene {
        EmlScene { asset }
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "f8d22a65-d671-4fa6-ae8f-0dccdb387ddd"]
pub struct EmlAsset {
    root: Arc<EmlElement>,
}

impl EmlAsset {
    pub fn write(&self, world: &mut World, parent: Entity) {
        walk(&self.root, world, Some(parent));
    }
}

fn walk(node: &EmlElement, world: &mut World, parent: Option<Entity>) -> Option<Entity> {
    let Some(builder) = world
        .resource::<ElementBuilderRegistry>()
        .get_builder(node.name)
    else {
        error!("Invalid tag name: {}", node.name.as_str());
        return None;
    };
    let entity = parent.unwrap_or_else(|| world.spawn_empty().id());
    let mut context = ElementContextData::new(entity);
    for (name, value) in node.attributes.iter() {
        let attr = Attribute::new(name, value.clone().into());
        context.attributes.add(attr);
    }
    for child in node.children.iter() {
        match child {
            EmlNode::Text(text) => {
                let entity = world
                    .spawn(TextBundle {
                        text: Text::from_section(text, Default::default()),
                        ..default()
                    })
                    .insert(Element::inline())
                    .id();
                context.children.push(entity);
            }
            EmlNode::Element(child) => {
                if let Some(entity) = walk(child, world, None) {
                    context.children.push(entity);
                }
            }
        };
    }
    builder.build(world, context);
    Some(entity)
}

#[derive(Default)]
pub(crate) struct EmlLoader {
    pub(crate) registry: ElementBuilderRegistry,
    pub(crate) validator: PropertyValidator,
    pub(crate) extractor: PropertyExtractor,
}

impl AssetLoader for EmlLoader {
    fn extensions(&self) -> &[&str] {
        &["eml"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let source = std::str::from_utf8(bytes)?;

            match parse::parse(source, self) {
                Ok(root) => {
                    let asset = EmlAsset {
                        root: Arc::new(root),
                    };
                    load_context.set_default_asset(LoadedAsset::new(asset));
                    Ok(())
                }
                Err(err) => {
                    let path = load_context.path();
                    error!("Error parsing {}:\n\n{}", path.to_str().unwrap(), err);
                    Err(bevy::asset::Error::new(err)
                        .context(format!("Unable to parse {}", path.to_str().unwrap())))
                }
            }
        })
    }
}

pub fn update_eml_scene(
    scenes: Query<(Entity, &EmlScene, Option<&Children>)>,
    mut events: EventReader<AssetEvent<EmlAsset>>,
    assets: Res<Assets<EmlAsset>>,
    mut commands: Commands,
) {
    for event in events.iter() {
        if let AssetEvent::Created { handle } = event {
            let asset = assets.get(handle).unwrap();
            for (entity, _, _) in scenes.iter().filter(|(_, s, _)| &s.asset == handle) {
                let asset = asset.clone();
                commands.add(move |world: &mut World| {
                    asset.write(world, entity);
                });
            }
        } else if let AssetEvent::Modified { handle } = event {
            let asset = assets.get(handle).unwrap();
            for (entity, _, children) in scenes.iter().filter(|(_, s, _)| &s.asset == handle) {
                if let Some(children) = children {
                    for ch in children.iter() {
                        commands.entity(*ch).despawn_recursive();
                    }
                }
                let asset = asset.clone();
                commands.add(move |world: &mut World| {
                    asset.write(world, entity);
                });
            }
        }
    }
}
