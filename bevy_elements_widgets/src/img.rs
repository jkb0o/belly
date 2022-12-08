use bevy::{prelude::*, utils::HashMap};
use bevy_elements_core::*;
use bevy_elements_macro::*;

pub(crate) struct ImgPlugin;
impl Plugin for ImgPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<Img>();

        app.init_resource::<ImageRegistry>();
        app.add_system(load_img);
        app.add_system(update_img_size);
        app.add_system(update_img_layout);
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ImageRegistry(HashMap<Handle<Image>, Entity>);

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum ImgMode {
    #[default]
    Fit,
    Cover,
    Stretch,
    Source,
}

impl TryFrom<Variant> for ImgMode {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::String(s) if &s == "fit" => Ok(ImgMode::Fit),
            Variant::String(s) if &s == "cover" => Ok(ImgMode::Cover),
            Variant::String(s) if &s == "stretch" => Ok(ImgMode::Stretch),
            Variant::String(s) if &s == "source" => Ok(ImgMode::Source),
            Variant::String(s) => Err(format!("Can't parse `{}` as ImgMode", s)),
            variant => {
                if let Some(value) = variant.take::<ImgMode>() {
                    Ok(value)
                } else {
                    Err("Invalid value for ImgMode".to_string())
                }
            }
        }
    }
}

impl From<ImgMode> for Variant {
    fn from(mode: ImgMode) -> Self {
        Variant::Any(Box::new(mode))
    }
}

#[derive(Component, Widget)]
#[alias(img)]
/// The `<img>` tag is used to load image and show it content on the UI screen.
/// The `<img>` tag has two properties:
/// - `src`: Specifies the path to the image
/// - `mode`: Specifies how an image should fits the space:
///   - `fit`: resize the image to fit the box keeping it aspect ratio
///   - `cover`: resize the image to cover the box keeping it aspect ratio
///   - `stretch`: resize image to take all the space ignoring the aspect ratio
///   - `source`: do not resize the image
pub struct Img {
    #[param]
    pub src: String,
    #[param]
    pub mode: ImgMode,
    entity: Entity,
    size: Vec2,
}

impl WidgetBuilder for Img {
    fn setup(&mut self, ctx: &mut ElementContext) {
        ctx.commands().entity(self.entity).insert(ImageBundle {
            style: Style {
                display: Display::None,
                ..default()
            },
            ..default()
        });
        ctx.insert(ElementBundle::default())
            .push_children(&[self.entity]);
    }
}

fn load_img(
    asset_server: Res<AssetServer>,
    mut elements: Query<(Entity, &mut Img), Changed<Img>>,
    mut images: Query<(&mut UiImage, &mut Style)>,
    mut registry: ResMut<ImageRegistry>,
    assets: Res<Assets<Image>>,
    mut events: EventWriter<AssetEvent<Image>>,
) {
    for (entity, mut img) in elements.iter_mut() {
        let handle = asset_server.load(&img.src);
        registry.insert(handle.clone_weak(), entity);
        let (mut image, mut style) = images.get_mut(img.entity).unwrap();
        image.0 = handle.clone();

        // force inner image size recalculation if Image asset already loaded
        if assets.contains(&handle) {
            style.display = Display::Flex;
            events.send(AssetEvent::Modified {
                handle: handle.clone_weak(),
            });
        } else {
            if img.size != Vec2::ZERO {
                img.size = Vec2::ZERO;
            }
            style.display = Display::None;
        }
    }
}

fn update_img_size(
    mut elements: Query<&mut Img>,
    assets: Res<Assets<Image>>,
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut registry: ResMut<ImageRegistry>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Removed { handle } => {
                let Some(entity) = registry.remove(&handle) else { continue };
                let Ok(mut element) = elements.get_mut(entity) else { continue };
                element.size = Vec2::ZERO;
            }
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let Some(entity) = registry.get(&handle) else { continue };
                let Ok(mut element) = elements.get_mut(*entity) else { continue };
                let Some(asset) = assets.get(handle) else { continue };
                if element.size != asset.size() {
                    element.size = asset.size();
                }
            }
        }
    }
}

fn update_img_layout(
    elements: Query<(&Img, &Node), Or<(Changed<Img>, Changed<Node>)>>,
    mut styles: Query<&mut Style>,
) {
    for (element, node) in elements.iter() {
        let Ok(mut style) = styles.get_mut(element.entity) else { continue };
        if element.size.x.abs() < f32::EPSILON
            || element.size.y.abs() < f32::EPSILON
            || node.size().x.abs() < f32::EPSILON
            || node.size().y.abs() < f32::EPSILON
        {
            style.display = Display::None;
            continue;
        } else {
            style.display = Display::Flex;
        }
        let aspect = element.size.y / element.size.x;
        match element.mode {
            ImgMode::Fit => {
                let (width, height) = if aspect > 1.0 {
                    let width = node.size().x;
                    let height = width * aspect;
                    if height > node.size().y {
                        let width = width * (node.size().y / height);
                        let height = node.size().y;
                        (width, height)
                    } else {
                        (width, height)
                    }
                } else {
                    let height = node.size().y;
                    let width = height / aspect;
                    if width > node.size().x {
                        let height = height * (node.size().x / width);
                        let width = node.size().x;
                        (width, height)
                    } else {
                        (width, height)
                    }
                };
                style.min_size.height = Val::Px(height);
                style.min_size.width = Val::Px(width);
                style.size = style.min_size;
                let hmargin = 0.5 * (node.size().x - width);
                let vmargin = 0.5 * (node.size().y - height);

                style.margin.top = Val::Px(vmargin.max(0.));
                style.margin.bottom = Val::Px(vmargin.max(0.));
                style.margin.left = Val::Px(hmargin.max(0.));
                style.margin.right = Val::Px(hmargin.max(0.));
            }
            ImgMode::Cover => {
                let (width, height) = if aspect > 1.0 {
                    let width = node.size().x;
                    let height = width * aspect;
                    if height < node.size().y {
                        let width = width * (node.size().y / height);
                        let height = node.size().y;
                        (width, height)
                    } else {
                        (width, height)
                    }
                } else {
                    let height = node.size().y;
                    let width = height / aspect;
                    if width < node.size().x {
                        let height = height * (node.size().x / width);
                        let width = node.size().x;
                        (width, height)
                    } else {
                        (width, height)
                    }
                };

                style.min_size.height = Val::Px(height);
                style.min_size.width = Val::Px(width);
                style.size = style.min_size;
                let hmargin = 0.5 * (node.size().x - width);
                let vmargin = 0.5 * (node.size().y - height);

                style.margin.top = Val::Px(vmargin.min(0.));
                style.margin.bottom = Val::Px(vmargin.min(0.));
                style.margin.left = Val::Px(hmargin.min(0.));
                style.margin.right = Val::Px(hmargin.min(0.));
            }
            ImgMode::Stretch => {
                style.min_size = Size::new(Val::Undefined, Val::Undefined);
                style.size = Size::new(Val::Percent(100.), Val::Percent(100.));
                style.margin = UiRect::all(Val::Px(0.));
            }
            ImgMode::Source => {
                style.size = Size::new(Val::Px(element.size.x), Val::Px(element.size.y));
                style.min_size = style.size;
                let hmargin = 0.5 * (node.size().x - element.size.x);
                let vmargin = 0.5 * (node.size().y - element.size.y);
                style.margin.left = Val::Px(hmargin);
                style.margin.right = Val::Px(hmargin);
                style.margin.top = Val::Px(vmargin);
                style.margin.bottom = Val::Px(vmargin);
            }
        }
    }
}
