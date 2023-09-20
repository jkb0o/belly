use belly::build::*;
use belly_core::eml::Eml;
use bevy::{
    ecs::{
        component::Tick,
        query::WorldQuery,
        system::{CommandQueue, SystemMeta, SystemParam},
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    prelude::*,
    utils::HashMap,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        // this will be implemented inside belly plugins
        .add_event::<DatabaseEvent>()
        .add_systems(PreUpdate, init_lists_system)
        .add_systems(PreUpdate, process_events_system)
        .run();
}

// this is how example will look like

pub struct TaskData {
    name: String,
    complete: bool,
}

impl TaskData {
    pub fn complete_label(&self) -> &'static str {
        if self.complete {
            "Completed"
        } else {
            "Complete"
        }
    }
}
// aliases to make queries, binds and callbacks more explict
type Tasks<'w, 's> = Database<'w, 's, TaskData>;
type Task = Item<TaskData>;

fn setup(mut commands: Commands, mut db: Tasks) {
    commands.spawn(Camera2dBundle::default());

    let todo = db
        .add_collection()
        .push(TaskData {
            name: "hello".into(),
            complete: false,
        })
        .push(TaskData {
            name: "".into(),
            complete: true,
        })
        .id();

    commands.add(eml! {
        <body>
            <list collection=todo render-item=|item| eml! {
                <span c:task>
                    <textinput
                        bind:value=from!(item, Task:name)
                        bind:value=to!(item, Task:name)
                    />
                    <button mode="toggle"
                        bind:pressed=from!(item, Task:complete)
                        bind:pressed=to!(item, Task:complete)
                    >
                        {from!(item, Task:complete_label())}
                    </button>
                </span>
            }/>

            // the right way to implement add button lokks like this:

            // <button on:press=run!(|tasks: Tasks| {
            //     tasks.collection(todo).push(TaskData {
            //         name: format!("Yet another task"),
            //         complete: false
            //     })
            // })>"Add task"</button>

            // but it is not possible yet to accept SystemParam
            // as run! closure argument, only WorldQuery args work.
            // There is the version with EventContext push_item
            // extension:
            <button on:press=run!(|ctx| {
                ctx.push_item(todo, TaskData {
                    name: format!("Yet another task"),
                    complete: false
                });
            })>"Add task"</button>
        </body>
    });

    commands.add(ess! {
        body {
            padding: 50px;
            flex-direction: column;
            background-color: white;
            align-content: center;
            align-items: center;
        }
        list {
            flex-direction: column;
            align-content: center;
            align-items: center;
        }
        .task {
            align-content: center;
            align-items: center;
        }
        .task textinput {
            height: 30px;
            width: 300px;
        }
        .task button {
            width: 150px;
        }
    })
}

// the rest is the implementation:
// - Added `Collection` and `Item` component
// - Added `Database` `SystemParam`
// - Added `<list>` widget with required systems
// - (temporary) Added EventContextCollectionExtension
//   to be able to call `ctx.push_item(collection, value)`
//   from callbacks

/// `Collection` is the data component holding single item of data.
#[derive(Component)]
pub struct Collection;

/// `Item` is the
#[derive(Component, Deref, DerefMut)]
pub struct Item<T>(T);

#[derive(Event, Debug)]
pub enum DatabaseEvent {
    ItemAdded { collection: Entity, item: Entity },
    ItemRemoved { collection: Entity, item: Entity },
}

#[derive(SystemParam)]
pub struct Database<'w, 's, T: 'static + Send + Sync> {
    lists: Query<'w, 's, QCollections, ()>,
    items: Query<'w, 's, QItems<T>, ()>,
    commands: DatabaseCommands<'w, 's>,
    events: EventWriter<'w, DatabaseEvent>,
}

impl<'w, 's, T: 'static + Send + Sync> Database<'w, 's, T> {
    pub fn add_collection<'d>(&'d mut self) -> CollectionRef<'w, 's, 'd, T> {
        let entity = self.commands.spawn(Collection).id();
        self.collection(entity)
    }
    pub fn collection<'d>(&'d mut self, entity: Entity) -> CollectionRef<'w, 's, 'd, T> {
        CollectionRef {
            root: entity,
            db: self,
        }
    }
}

#[derive(WorldQuery)]
pub struct QCollections {
    pub entity: Entity,
    list: &'static Collection,
    children: &'static Children,
}

#[derive(WorldQuery)]
pub struct QItems<T: 'static + Send + Sync> {
    item: &'static Item<T>,
}

#[derive(Deref, DerefMut)]
pub struct DatabaseCommands<'w, 's>(Commands<'w, 's>);

impl<'w, 's> DatabaseCommands<'w, 's> {
    pub fn new(queue: &'s mut CommandQueue, world: &'w World) -> Self {
        Self(Commands::new(queue, world))
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct DatabaseCommandsQueue(CommandQueue);

// SAFETY: Commands only accesses internal state
unsafe impl<'w, 's> SystemParam for DatabaseCommands<'w, 's> {
    type State = DatabaseCommandsQueue;
    type Item<'world, 'state> = DatabaseCommands<'world, 'state>;

    fn init_state(_world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        Default::default()
    }

    #[inline]
    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        _system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        _change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        DatabaseCommands::new(&mut state.0, world.world())
    }

    fn apply(state: &mut Self::State, _system_meta: &SystemMeta, world: &mut World) {
        state.0.apply(world);
    }
}

pub struct CollectionRef<'w, 's, 'd, T: 'static + Send + Sync> {
    db: &'d mut Database<'w, 's, T>,
    root: Entity,
}

impl<'w, 's, 'd, T: 'static + Send + Sync> CollectionRef<'w, 's, 'd, T> {
    pub fn id(&self) -> Entity {
        self.root
    }
    pub fn push(&mut self, item: T) -> &mut Self {
        let item = self.db.commands.spawn(Item(item)).id();
        self.db.commands.entity(self.root).add_child(item);
        self.db.events.send(DatabaseEvent::ItemAdded {
            collection: self.root,
            item,
        });
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let items = self.db.lists.get(self.root).unwrap().children;
        let mut iter = self.db.items.iter_many(items);
        std::iter::from_fn(move || iter.next().map(|i| &i.item.0))
    }
}

pub trait EventContextCollectionExtension {
    fn push_item<T: 'static + Send + Sync>(&mut self, collection: Entity, item: T);
}

impl<'c, 'w, 's, E: Event> EventContextCollectionExtension for EventContext<'c, 'w, 's, E> {
    fn push_item<T: 'static + Send + Sync>(&mut self, collection: Entity, item: T) {
        let item = self.commands().spawn(Item(item)).id();
        self.commands().entity(collection).add_child(item);
        self.commands().add(move |world: &mut World| {
            let event = DatabaseEvent::ItemAdded { collection, item };
            world.resource_mut::<Events<DatabaseEvent>>().send(event);
        });
    }
}

pub type RenderFunction = Box<dyn Fn(Entity) -> Eml>;

#[derive(Component)]
pub struct List {
    collection: Entity,
    render_func: RenderFunction,
    rendered_items: HashMap<Entity, Entity>,
}

unsafe impl Send for List {}
unsafe impl Sync for List {}

#[widget]
fn list(ctx: &mut WidgetContext) {
    let render_func = ctx
        .param("render-item".into())
        .expect("`render-item` param is required for `<list/>`")
        .take::<RenderFunction>()
        .expect("`render-item` param is required for `<list/>`");
    let collection = ctx
        .param("collection".into())
        .expect("`collection` param is required for `<list/>`")
        .take::<Entity>()
        .expect("`collection` param is required for `<list/>`");
    info!("adding widget for collection {collection:?}");
    let list = List {
        collection,
        render_func,
        rendered_items: HashMap::default(),
    };
    ctx.render(eml! { <span with=list/> })
}

fn init_lists_system(
    mut elements: Elements,
    mut lists: Query<(Entity, &mut List), Added<List>>,
    collections: Query<&Children, With<Collection>>,
) {
    for (entity, mut list) in lists.iter_mut() {
        let Ok(items) = collections.get(list.collection) else {
            continue
        };
        for item in items.iter() {
            if !list.rendered_items.contains_key(item) {
                elements.entity(entity).add_child_with(|rendered| {
                    list.rendered_items.insert(*item, rendered);
                    (list.render_func)(*item)
                });
            }
        }
    }
}

fn process_events_system(
    mut elements: Elements,
    mut widgets: Query<(Entity, &mut List)>,
    mut events: EventReader<DatabaseEvent>,
) {
    for event in events.iter() {
        info!("event: {event:?}");
        for (entity, mut widget) in widgets.iter_mut() {
            info!("list with collection: {:?}", widget.collection);
            match event {
                DatabaseEvent::ItemAdded { collection, item } => {
                    // info!("item added, {:?}, {:?}, {:?}", widget.collection, )
                    if &widget.collection == collection && !widget.rendered_items.contains_key(item)
                    {
                        info!("adding child to {entity:?}");
                        elements.entity(entity).add_child_with(|rendered| {
                            widget.rendered_items.insert(*item, rendered);
                            (widget.render_func)(*item)
                        });
                    }
                }
                _ => (),
            }
        }
    }
}
