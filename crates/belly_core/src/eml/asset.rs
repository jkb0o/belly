use crate::element::Element;
use crate::eml::{
    build::{ElementBuilderRegistry, ElementContextData, Slots},
    parse, Param,
};
use crate::ess::{PropertyExtractor, PropertyTransformer};
use crate::relations::connect::ScriptHandler;
use bevy::ecs::system::Command;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use rhai::{Engine, Scope, AST};
use std::sync::{Arc, RwLock};
use tagstr::*;

pub struct EmlRoot {
    // I guess it will be a Vec<EmlScriptDeclaration>
    pub script: Option<EmlScriptDeclaration>,
    pub root: EmlNode,
}

pub enum EmlNode {
    Element(EmlElement),
    Text(String),
    Slot(Tag, Vec<EmlNode>),
}

#[derive(Default)]
pub struct EmlScriptDeclaration {
    pub source: String,
}

#[derive(Resource, Deref)]
/// Scripting engine here for example purposes
pub struct ScriptingEngine(Engine);
unsafe impl Send for ScriptingEngine {}
unsafe impl Sync for ScriptingEngine {}

#[derive(Resource, Default, Clone, Deref)]
/// Scripts here for example purposes
pub struct Scripts(Arc<RwLock<HashMap<Handle<EmlAsset>, AST>>>);
unsafe impl Send for Scripts {}
unsafe impl Sync for Scripts {}

#[derive(Default)]
pub struct EmlElement {
    pub(crate) name: Tag,
    pub(crate) params: HashMap<String, String>,
    pub(crate) connections: HashMap<String, String>,
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
    root: Arc<EmlRoot>,
}

impl EmlAsset {
    pub fn write(&self, world: &mut World, handle: Handle<EmlAsset>, parent: Entity) {
        walk(&self.root.root, world, handle, Some(parent));
    }
}

fn walk(
    node: &EmlNode,
    world: &mut World,
    handle: Handle<EmlAsset>,
    parent: Option<Entity>,
) -> Option<Entity> {
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
                .filter_map(|e| walk(e, world, handle.clone_weak(), None))
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

            // connect signals here
            for (signal, connection) in elem.connections.iter() {
                let connection = connection.clone();
                let handle = handle.clone_weak();
                builder.connect(
                    world,
                    entity,
                    signal,
                    ScriptHandler::new(move |world, source, _data| {
                        let scripts = world.get_resource_or_insert_with(Scripts::default).clone();
                        let scripts_ref = scripts.read().unwrap();
                        let Some(ast) = scripts_ref.get(&handle) else {
                        warn!("No script associated with eml asset");
                        return;
                    };
                        let Some(engine) = world.get_resource::<ScriptingEngine>() else {
                        warn!("No script engine registered");
                        return;
                    };
                        let mut scope = Scope::new();
                        let result = engine.call_fn::<()>(&mut scope, &ast, &connection, (source,));
                        if let Err(e) = result {
                            error!("Error calling method: {e:?}");
                        }
                    }),
                )
            }

            // build subtree
            for child in elem.children.iter() {
                if let Some(entity) = walk(child, world, handle.clone_weak(), None) {
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

pub struct AddScriptCommand {
    handle: Handle<EmlAsset>,
}
impl AddScriptCommand {
    pub fn new(handle: Handle<EmlAsset>) -> AddScriptCommand {
        AddScriptCommand { handle }
    }
}

impl Command for AddScriptCommand {
    fn write(self, world: &mut World) {
        if let Some(asset) = world
            .resource::<Assets<EmlAsset>>()
            .get(&self.handle)
            .cloned()
        {
            let engine = world.get_resource_or_insert_with(|| ScriptingEngine(Engine::new()));
            if let Some(script) = &asset.root.script {
                match engine.compile(&script.source) {
                    Err(e) => error!("Error compiling script: {e}"),
                    Ok(ast) => {
                        let scripts = world.get_resource_or_insert_with(Scripts::default).clone();
                        scripts
                            .write()
                            .unwrap()
                            .insert(self.handle.clone_weak(), ast);
                    }
                };
            }
        }
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
            commands.add(AddScriptCommand::new(handle.clone_weak()));
            let asset = assets.get(handle).unwrap();
            for (entity, _, _) in scenes.iter().filter(|(_, s, _)| &s.asset == handle) {
                let asset = asset.clone();
                let handle = handle.clone_weak();
                commands.add(move |world: &mut World| {
                    asset.write(world, handle, entity);
                });
            }
        } else if let AssetEvent::Modified { handle } = event {
            commands.add(AddScriptCommand::new(handle.clone_weak()));
            let asset = assets.get(handle).unwrap();
            for (entity, _, children) in scenes.iter().filter(|(_, s, _)| &s.asset == handle) {
                if let Some(children) = children {
                    for ch in children.iter() {
                        commands.entity(*ch).despawn_recursive();
                    }
                }
                let asset = asset.clone();
                let handle = handle.clone_weak();
                commands.add(move |world: &mut World| {
                    asset.write(world, handle, entity);
                });
            }
        }
    }
}
