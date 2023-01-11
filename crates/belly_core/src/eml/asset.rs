use crate::element::Element;
use crate::eml::{
    build::{ElementBuilderRegistry, ElementContextData, Slots},
    parse, Param,
};
use crate::ess::{PropertyExtractor, PropertyTransformer};
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use std::sync::Arc;
use tagstr::*;

pub enum EmlNode {
    Element(EmlElement),
    Text(String),
    Slot(Tag, Vec<EmlNode>),
}

#[derive(Default)]
pub struct EmlElement {
    pub(crate) name: Tag,
    pub(crate) params: HashMap<String, String>,
    pub(crate) children: Vec<EmlNode>,
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
    root: Arc<EmlNode>,
}

impl EmlAsset {
    pub fn write(&self, world: &mut World, parent: Entity) {
        // let node = E
        walk(&self.root, world, Some(parent));
    }
}

fn walk(node: &EmlNode, world: &mut World, parent: Option<Entity>) -> Option<Entity> {
    match node {
        EmlNode::Text(text) => {
            let entity = world
                .spawn(TextBundle {
                    text: Text::from_section(text, Default::default()),
                    ..default()
                })
                .insert(Element::inline())
                .id();
            Some(entity)
        }
        EmlNode::Slot(name, elements) => {
            let slots = world.resource::<Slots>().clone();
            let entities: Vec<Entity> = elements
                .iter()
                .filter_map(|e| walk(e, world, None))
                .collect();
            slots.insert(*name, entities);
            None
        }
        EmlNode::Element(elem) => {
            let Some(builder) = world
                .resource::<ElementBuilderRegistry>()
                .get_builder(elem.name)
            else {
                error!("Invalid tag name: {}", elem.name.as_str());
                return None;
            };
            let entity = parent.unwrap_or_else(|| world.spawn_empty().id());
            let mut context = ElementContextData::new(entity);
            for (name, value) in elem.params.iter() {
                let attr = Param::new(name, value.clone().into());
                context.params.add(attr);
            }
            for child in elem.children.iter() {
                if let Some(entity) = walk(child, world, None) {
                    context.children.push(entity);
                }
            }
            builder.build(world, context);
            Some(entity)
        }
    }
}

#[derive(Default)]
pub(crate) struct EmlLoader {
    pub(crate) registry: ElementBuilderRegistry,
    pub(crate) transformer: PropertyTransformer,
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
