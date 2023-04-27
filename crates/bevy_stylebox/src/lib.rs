use std::{any::type_name, marker::PhantomData};

/// The `bevy_stylebox` is plugin for [bevy](https://bevyengine.org/) engine which
/// allows you to fill UI node with sliced by 9 parts region of image. `Stylebox`
/// doesn't add any additional UI components. It renders just like `UiImage`, but
/// generates more vertices in the rendering system. Only `stretch` mode is
/// supported for now for drawing edges, `repeat` & `round` coming soon.
use bevy::{
    math::Rect,
    prelude::*,
    render::{Extract, RenderApp},
    ui::{ExtractedUiNode, ExtractedUiNodes, FocusPolicy, RenderUiSystem, UiStack},
    window::PrimaryWindow,
};

/// `Stylebox` plugin for `bevy` engine. Dont forget to register it:
/// ```rust
/// use bevy::prelude::*;
/// use bevy_stylebox::*;
///
/// fn main() {
///    let mut app = App::new();
///    app.add_plugins(DefaultPlugins);
///    app.add_plugin(StyleboxPlugin);
/// }
/// ```
pub struct StyleboxPlugin;

const EPSILON: f32 = 0.000005;
const TWO_EPSILONS: f32 = EPSILON + EPSILON;
const ONE_MINUS_EPSILON: f32 = 1.0 - EPSILON;
const ONE_MINUS_TWO_EPSILONS: f32 = ONE_MINUS_EPSILON - EPSILON;

impl Plugin for StyleboxPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                (
                    compute_covers_and_shadows,
                    compute_stylebox_configuration,
                    compute_stylebox_slices::<Shadow>
                        .after(compute_stylebox_configuration)
                        .after(compute_covers_and_shadows),
                    compute_stylebox_slices::<Cover>
                        .after(compute_stylebox_configuration)
                        .after(compute_covers_and_shadows),
                    compute_stylebox_slices::<Stylebox>
                        .after(compute_stylebox_configuration)
                        .after(compute_covers_and_shadows),
                )
                    .in_base_set(CoreSet::PostUpdate),
            )
            .sub_app_mut(RenderApp)
            .add_systems(
                (
                    extract_stylebox::<Shadow>,
                    extract_stylebox::<Cover>,
                    extract_stylebox::<Stylebox>,
                )
                    .chain()
                    .after(RenderUiSystem::ExtractNode)
                    .in_schedule(ExtractSchedule),
            );
    }
}
#[derive(Bundle, Clone, Debug, Default)]
/// The bundle with almost the same content a `NodeBundle`,
/// but `Stylebox` component is used instead of `BackgroundColor`.
pub struct StyleboxBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// The stylebox of the node
    pub stylebox: Stylebox,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    pub transform: Transform,
    /// The global transform of the node
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

#[derive(Component, Clone, Debug)]
/// Component used to specify how to fill the element with sliced by 9 parts region of image.
pub struct Stylebox {
    /// holds the handle to the image to be used as a stylebox
    pub texture: Handle<Image>,
    /// specifies how to slice the image region specified by texture & region
    /// The image is always sliced into nine sections: four corners, four edges and the middle.
    /// - when `Val::Px` specified, region sliced to the exact amount of pixels
    /// - when `Val::Percent` specified, image region sliced relative to it size
    /// - `Val::Auto` & `Val::Undefined` treated as `Val::Percent(50.)`
    pub slice: UiRect,
    /// specifies the width of the edgets of the sliced region:
    /// - edges specified by `Val::Px` values resizes to exact amout of pixels
    /// - edges specified by `Val::Percent` resized relative to width provided by `slice` property
    /// - `Val::Auto` & `Val::Undefined` treated as `Val::Percent(100.)`
    ///
    /// Default value for `width` is `Val::Percent(100.)`: use width provided by `slice` property.
    pub width: UiRect,
    /// specifies which region of the image should be sliced.
    /// By default the hole area of image defined by `texture` is used.
    /// - `Val::Px` values defines exact offset from the image edges in pixels
    /// - `Val::Percent` values defines offset from the image edges relative to the image size
    /// - `Val::Auto` & `Val::Undefined` treated as `Val::Px(0.)`
    ///
    /// Default value for `region` is `Val::Px(0.)`
    pub region: UiRect,
    /// specifies what color the original image should be multiplied by
    pub modulate: Color,
}

impl Default for Stylebox {
    fn default() -> Self {
        Stylebox {
            texture: Handle::<Image>::default(),
            slice: UiRect::default(),
            width: UiRect::all(Val::Percent(100.)),
            modulate: Color::WHITE,
            region: UiRect::all(Val::Px(0.)),
        }
    }
}

impl std::fmt::Display for Stylebox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stylebox (\n  slice: {:?},\n  width: {:?},\n  region: {:?}\n)",
            self.slice, self.width, self.region
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct UiRectF32 {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl UiRectF32 {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> UiRectF32 {
        UiRectF32 {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.right + self.left, self.bottom + self.top)
    }
}

#[derive(Component, Default)]
/// Component which holds calculated sizes based on `slice`, `width` and `region` provided
/// by `StyleBox`.
pub struct ComputedStylebox {
    // slice rect relative to
    // region size in 0..1 space
    slice: UiRectF32,

    // edges width relative to slices edges
    width: UiRectF32,
    region: Rect,
}

#[derive(Component)]
pub struct StyleboxTexture(Handle<Image>);

/// Calculates exact values for `Stylebox` when image size is available and stores it into `ComputedStylebox`.
pub fn compute_stylebox_configuration(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    mut styleboxes: Query<
        (
            Entity,
            &Stylebox,
            Option<&mut ComputedStylebox>,
            Option<&mut StyleboxTexture>,
        ),
        Or<(Changed<Stylebox>, Without<ComputedStylebox>)>,
    >,
) {
    for (entity, stylebox, computed, texture) in styleboxes.iter_mut() {
        info!("compute_stylebox_configuration for {entity:?}");
        if stylebox.texture == Handle::<Image>::default() {
            if computed.is_none() {
                commands
                    .entity(entity)
                    .insert(StyleboxSlices::<Stylebox>::default())
                    .insert(StyleboxTexture(stylebox.texture.clone()))
                    .insert(ComputedStylebox::default());
                info!("inserting empty texture");
                continue;
            }
        }
        match images.get(&stylebox.texture) {
            None => {
                if computed.is_some() {
                    commands
                        .entity(entity)
                        .remove::<StyleboxSlices<Stylebox>>()
                        .remove::<ComputedStylebox>();
                }
                if texture.is_some() {
                    info!("removing texture");
                    commands.entity(entity).remove::<StyleboxTexture>();
                }
            }
            Some(image) => {
                let size = image.size();
                let region_left = match stylebox.region.left {
                    Val::Percent(percent) => size.x * percent * 0.01,
                    Val::Px(px) => px,
                    _ => 0.,
                };
                let region_right = match stylebox.region.right {
                    Val::Percent(percent) => size.x * percent * 0.01,
                    Val::Px(px) => px,
                    _ => 0.,
                };
                let region_top = match stylebox.region.top {
                    Val::Percent(percent) => size.y * percent * 0.01,
                    Val::Px(px) => px,
                    _ => 0.,
                };
                let region_bottom = match stylebox.region.bottom {
                    Val::Percent(percent) => size.y * percent * 0.01,
                    Val::Px(px) => px,
                    _ => 0.,
                };
                let region = Rect {
                    min: Vec2::new(region_left, region_top),
                    max: Vec2::new(
                        (size.x - region_right).max(region_left),
                        (size.y - region_bottom).max(region_top),
                    ),
                };
                let size = region.size();

                let slice_left = match stylebox.slice.left {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.x,
                    _ => 0.5,
                };
                let slice_right = match stylebox.slice.right {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.x,
                    _ => 0.5,
                };
                let slice_top = match stylebox.slice.top {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.y,
                    _ => 0.5,
                };
                let slice_bottom = match stylebox.slice.bottom {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.y,
                    _ => 0.5,
                };
                let slice = UiRectF32::new(slice_left, slice_right, slice_top, slice_bottom);

                let width_left = match stylebox.width.left {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.x * slice.left),
                    _ => 1.0,
                };
                let width_right = match stylebox.width.right {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.x * slice.right),
                    _ => 1.0,
                };
                let width_top = match stylebox.width.top {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.y * slice.top),
                    _ => 1.0,
                };
                let width_bottom = match stylebox.width.bottom {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.y * slice.bottom),
                    _ => 1.0,
                };
                let width = UiRectF32::new(width_left, width_right, width_top, width_bottom);

                if let Some(mut computed) = computed {
                    computed.region = region;
                    computed.slice = slice;
                    computed.width = width;
                } else {
                    commands
                        .entity(entity)
                        .insert(StyleboxSlices::<Stylebox>::default())
                        .insert(ComputedStylebox {
                            region,
                            slice,
                            width,
                        });
                }

                if let Some(mut texture) = texture {
                    texture.0 = stylebox.texture.clone();
                    info!("updating texture");
                } else {
                    info!("inserting texture");
                    commands
                        .entity(entity)
                        .insert(StyleboxTexture(stylebox.texture.clone()));
                }
            }
        }
    }
}

pub trait StyleboxOperations: Component + std::fmt::Debug {
    fn extend(&self) -> UiRectF32;
    fn translate(&self) -> UiRectF32;
    fn modulate(&self) -> Color;
}

impl StyleboxOperations for Stylebox {
    fn extend(&self) -> UiRectF32 {
        UiRectF32::default()
    }
    fn translate(&self) -> UiRectF32 {
        UiRectF32::default()
    }
    fn modulate(&self) -> Color {
        self.modulate
    }
}

#[derive(Component, Debug, Default)]
pub struct Cover {
    pub width: UiRect,
    pub modulate: Color,
    extend: UiRectF32,
}
impl Cover {
    pub fn new(width: UiRect, modulate: Color) -> Self {
        Self {
            width, modulate, extend: UiRectF32::default()
        }
    }
}
impl StyleboxOperations for Cover {
    fn extend(&self) -> UiRectF32 {
        self.extend
    }
    fn translate(&self) -> UiRectF32 {
        UiRectF32::default()
    }
    fn modulate(&self) -> Color {
        self.modulate
    }
}

#[derive(Component, Debug, Default)]
pub struct Shadow {
    pub offset: UiRect,
    pub modulate: Color,
    translate: UiRectF32,
    extend: UiRectF32,
}
impl Shadow {
    pub fn new(offset: UiRect, modulate: Color) -> Shadow {
        Shadow { offset, modulate, ..default() }
    }
}
impl StyleboxOperations for Shadow {
    fn extend(&self) -> UiRectF32 {
        self.extend
    }
    fn translate(&self) -> UiRectF32 {
        self.translate
    }
    fn modulate(&self) -> Color {
        self.modulate
    }
}

pub fn compute_covers_and_shadows(
    mut commands: Commands,
    mut nodes: Query<
        (
            Entity, 
            &Node, 
            Option<&mut Cover>,
            Option<&mut Shadow>,
            Option<&StyleboxSlices<Cover>>,
            Option<&StyleboxSlices<Shadow>>,
        ),
        Or<(Changed<Node>, Changed<Cover>, Changed<Shadow>)>,
    >,
) {
    for (entity, node, cover, shadow, cover_slices, shadow_slices) in nodes.iter_mut() {
        let mut size = node.size();
        let mut extend = UiRectF32::default();
        if let Some(mut cover) = cover {
            let left = match cover.width.left {
                Val::Px(v) => v,
                Val::Percent(p) => size.x * p,
                _ => 0.,
            };
            let right = match cover.width.right {
                Val::Px(v) => v,
                Val::Percent(p) => size.x * p,
                _ => 0.,
            };
            let top = match cover.width.top {
                Val::Px(v) => v,
                Val::Percent(p) => size.y * p,
                _ => 0.,
            };
            let bottom = match cover.width.bottom {
                Val::Px(v) => v,
                Val::Percent(p) => size.y * p,
                _ => 0.,
            };
            size.x += left + right;
            size.y += top + bottom;
            extend = UiRectF32::new(left, right, top, bottom);
            if cover.extend != extend {
                cover.extend = extend;
            }
            if cover_slices.is_none() {
                commands.entity(entity).insert(StyleboxSlices::<Cover>::default());
            }
        } else {
            commands.entity(entity).remove::<StyleboxSlices<Cover>>();
        }
        if let Some(mut shadow) = shadow {
            let left = -1. * match shadow.offset.left {
                    Val::Px(v) => v,
                    Val::Percent(p) => size.x * p,
                    _ => 0.,
                };
            let right = match shadow.offset.right {
                    Val::Px(v) => v,
                    Val::Percent(p) => size.x * p,
                    _ => 0.,
                };
            let top = -1. * match shadow.offset.top {
                    Val::Px(v) => v,
                    Val::Percent(p) => size.y * p,
                    _ => 0.,
                };
            let bottom = match shadow.offset.bottom {
                    Val::Px(v) => v,
                    Val::Percent(p) => size.y * p,
                    _ => 0.,
                };
            let translate = UiRectF32::new(left, right, top, bottom);
            if shadow.translate != translate {
                shadow.translate = translate;
            }

            if shadow.extend != extend {
                shadow.extend = extend;
            }
            if shadow_slices.is_none() {
                commands.entity(entity).insert(StyleboxSlices::<Shadow>::default());
            }
        } else {
            commands.entity(entity).remove::<StyleboxSlices::<Shadow>>();
        }
    }
}

struct StyleboxSlice {
    transform: Mat4,
    region: Rect,
}

#[derive(Component, Default)]
/// Holds transform for each slice
pub struct StyleboxSlices<T> {
    layer: PhantomData<T>,
    items: Vec<StyleboxSlice>,
}
/// Calculates transforms for each slice based on `Node.size()` and `ComputedStylebox`
pub fn compute_stylebox_slices<Operation: StyleboxOperations>(
    mut query: Query<
        (Entity, &mut StyleboxSlices<Operation>, &Node, &ComputedStylebox, &Operation),
        Or<(Changed<Node>, Changed<ComputedStylebox>, Changed<Operation>)>,
    >,
) {
    for (entity, mut slices, uinode, stylebox, op) in query.iter_mut() {
        if uinode.size() == Vec2::ZERO {
            info!("zero-sized!");
            continue;
        }
        
        let extend = op.extend();
        
        let tr = op.translate();
        slices.items.clear();
        let size = uinode.size() + extend.size() + tr.size();
        // info!("computing slices for {}, extend: {:?}, orig-size: {:?}, size: {:?}", type_name::<Operation>(), extend, uinode.size(), size);
        let region = stylebox.region;
        let rpos = region.min;
        let rsize = region.size();

        let left = stylebox.slice.left;
        let right = stylebox.slice.right;
        let top = stylebox.slice.top;
        let bot = stylebox.slice.bottom;

        // compute part sizes in uinode space
        let w0 = left * rsize.x * stylebox.width.left + extend.left;
        let w2 = right * rsize.x * stylebox.width.right + extend.right;
        let w1 = size.x - w0 - w2;
        let h0 = top * rsize.y * stylebox.width.top + extend.top;
        let h2 = bot * rsize.y * stylebox.width.bottom + extend.bottom;
        let h1 = size.y - h0 - h2;

        let tr_h = 0.5 * (tr.right - tr.left);
        let tr_v = 0.5 * (tr.bottom - tr.top);

        let ui_x = &[tr_h, w0 + tr_h, w0 + w1 + tr_h];
        let ui_y = &[tr_v, h0 + tr_v, h0 + h1 + tr_v];
        let ui_width = &[w0, w1, w2];
        let ui_height = &[h0, h1, h2];
        let n = type_name::<Operation>();
        info!("{n} ui_x: {:?}", ui_x);
        info!("{n} ui_y: {:?}", ui_y);
        info!("{n} ui_width: {:?}", ui_width);
        info!("{n} ui_height: {:?}", ui_height);

        // make sure there is a minimum gap betwenn 0, left, right and 1
        let (left, right) = normalize_axis(left, right);
        let (top, bot) = normalize_axis(top, bot);

        // compute sizes in image space
        let img_x = &[
            rpos.x,
            rpos.x + left * rsize.x,
            rpos.x + (1. - right) * rsize.x,
        ];
        let img_y = &[
            rpos.y,
            rpos.y + top * rsize.y,
            rpos.y + (1. - bot) * rsize.y,
        ];
        let img_width = &[
            left * rsize.x,
            (1. - right - left) * rsize.x,
            right * rsize.x,
        ];
        let img_height = &[top * rsize.y, (1. - bot - top) * rsize.y, bot * rsize.y];

        for row in 0..3 {
            for col in 0..3 {
                if ui_width[row] < EPSILON || ui_height[col] < EPSILON {
                    continue;
                }
                let uirect = Rect {
                    min: Vec2::new(ui_x[row], ui_y[col]),
                    max: Vec2::new(ui_x[row] + ui_width[row], ui_y[col] + ui_height[col]),
                };

                let imgrect = Rect {
                    min: Vec2::new(img_x[row], img_y[col]),
                    max: Vec2::new(img_x[row] + img_width[row], img_y[col] + img_height[col]),
                };

                let center = 0.5 * (uirect.min + uirect.max);
                let offset = center - size * 0.5;
                let scale = uirect.size() / imgrect.size();
                // info!("scale: {scale:?}, offset: {offset:?}");
                let mut tr = Mat4::IDENTITY;
                tr *= Mat4::from_translation(offset.extend(0.));
                tr *= Mat4::from_scale(scale.extend(1.));
                slices.items.push(StyleboxSlice {
                    transform: tr,
                    region: imgrect,
                });
            }
        }
        info!("compute_stylebox_slices<{}> for {entity:?}, size: {size:?}, elements: {}", type_name::<Operation>(), slices.items.len());
    }
}

fn normalize_axis(left: f32, right: f32) -> (f32, f32) {
    let mut x0 = left;
    let mut x1 = 1. - right;

    if x0 > x1 {
        x0 = x1;
    }

    x0 = x0.max(EPSILON);
    x1 = x1.min(ONE_MINUS_EPSILON);

    if x0 >= ONE_MINUS_TWO_EPSILONS && x1 >= ONE_MINUS_TWO_EPSILONS {
        x0 = ONE_MINUS_TWO_EPSILONS;
        x1 = ONE_MINUS_EPSILON;
    } else if x0 <= TWO_EPSILONS && x1 <= TWO_EPSILONS {
        x0 = EPSILON;
        x1 = TWO_EPSILONS;
    } else if (x1 - x0) < EPSILON && x0 >= 0.5 {
        x1 += EPSILON
    } else if (x1 - x0) < EPSILON && x0 < 0.5 {
        x0 -= EPSILON
    }
    (x0, 1. - x1)
}

/// Extracts stylebox vertices into render pipeline based on `Stylebox.texture`,
/// `Stylebox.modulate` and `StyleboxSlices`
pub fn extract_stylebox<Operation: StyleboxOperations>(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    ui_stack: Extract<Res<UiStack>>,
    windows: Extract<Query<&Window, With<PrimaryWindow>>>,
    images: Extract<Res<Assets<Image>>>,
    uinode_query: Extract<
        Query<(
            &Node,
            &GlobalTransform,
            &StyleboxTexture,
            &StyleboxSlices<Operation>,
            &Operation,
            &ComputedVisibility,
            Option<&CalculatedClip>,
        )>,
    >,
) {
    let Ok(primary) = windows.get_single() else {
        info!("no window");
        return;
    };
    let scale_factor = primary.scale_factor() as f32;
    for (stack_index, entity) in ui_stack.uinodes.iter().enumerate() {
        let Ok((uinode, transform, texture, slices, operation, visibility, clip)) = uinode_query.get(*entity) else {
            continue
        };
        if !visibility.is_visible() {
            continue;
        }
        let image = texture.0.clone_weak();
        // Skip unloaded images
        if !images.contains(&image) {
            continue;
        }

        if uinode.size() == Vec2::ZERO {
            info!("zero size");
            continue;
        }
        // info!("extracting {}, {entity:?}", type_name::<Operation>());

        let img = images.get(&image).unwrap();
        let tr = transform.compute_matrix();
        for patch in slices.items.iter() {
            extracted_uinodes.uinodes.push(ExtractedUiNode {
                transform: tr
                    * GlobalTransform::from_scale(Vec3::splat(scale_factor.recip()))
                        .compute_matrix()
                    * patch.transform,
                color: operation.modulate(),
                rect: patch.region,
                image: image.clone(),
                atlas_size: Some(img.size()),
                clip: clip.map(|clip| clip.clip),
                stack_index,
                flip_x: false,
                flip_y: false,
            });
        }
    }
}
