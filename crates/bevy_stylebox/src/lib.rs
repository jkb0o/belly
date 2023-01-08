#![doc = include_str!("../README.md")]

use bevy::{
    math::Rect,
    prelude::*,
    render::{Extract, RenderApp, RenderStage},
    ui::{ExtractedUiNode, ExtractedUiNodes, FocusPolicy, RenderUiSystem, UiStack},
    window::WindowId,
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
        app.add_system(compute_stylebox_configuration)
            .add_system_to_stage(CoreStage::PostUpdate, compute_stylebox_slices)
            .sub_app_mut(RenderApp)
            .add_system_to_stage(
                RenderStage::Extract,
                extract_stylebox.after(RenderUiSystem::ExtractNode),
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
    /// Only `Val::Px` and `Val::Percent` variants are supported:
    /// - when `Val::Px` specified, region sliced to the exact amount of pixels
    /// - when `Val::Percent` specified, image region sliced relative to it size
    pub slice: UiRect,
    /// specifies the width of the edgets of the sliced region.
    /// Only `Val::Px` and `Val::Percent` variants are supported:
    /// - edges specified by `Val::Px` values resizes to exact amout of pixels
    /// - edges specified by `Val::Percent` resized relative to width provided by `slice` property
    ///
    /// Default value for `width` is `Val::Percent(100.)`: use width provided by `slice` property.
    pub width: UiRect,
    /// specifies which region of the image should be sliced.
    /// By default the hole area of image defined by `texture` is used.
    /// Only `Val::Px` and `Val::Percent` variants are supported:
    /// - `Val::Px` values defines exact offset from the image edges in pixels
    /// - `Val::Percent` values defines offset from the image edges relative to the image size
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

#[derive(Default)]
struct UiRectF32 {
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
/// Calculates exact values for `Stylebox` when image size is available and stores it into `ComputedStylebox`.
pub fn compute_stylebox_configuration(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    images: Res<Assets<Image>>,
    mut styleboxes: Query<
        (Entity, &Stylebox, Option<&mut ComputedStylebox>),
        Or<(Changed<Stylebox>, Without<ComputedStylebox>)>,
    >,
) {
    for (entity, stylebox, computed) in styleboxes.iter_mut() {
        let mut cleanup = |msg| {
            let path = asset_server.get_handle_path(&stylebox.texture);
            let path = if path.is_none() {
                "<unknown>".to_string()
            } else {
                path.unwrap().path().display().to_string()
            };
            error!("Invalid NinePatch for {}: {}.\n{}", path, msg, stylebox);
            commands
                .entity(entity)
                .remove::<Stylebox>()
                .remove::<StyleboxSlices>()
                .remove::<ComputedStylebox>();
        };
        if stylebox.texture == Handle::<Image>::default() {
            if computed.is_none() {
                commands
                    .entity(entity)
                    .insert(StyleboxSlices::default())
                    .insert(ComputedStylebox::default());
                continue;
            }
        }
        match images.get(&stylebox.texture) {
            None => {
                if computed.is_some() {
                    commands
                        .entity(entity)
                        .remove::<StyleboxSlices>()
                        .remove::<ComputedStylebox>();
                }
            }
            Some(image) => {
                let size = image.size();
                let region_left = match stylebox.region.left {
                    Val::Percent(percent) => size.x * percent * 0.01,
                    Val::Px(px) => px,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the region.left: {:?}",
                            value
                        ))
                    }
                };
                let region_right = match stylebox.region.right {
                    Val::Percent(percent) => size.x * percent * 0.01,
                    Val::Px(px) => px,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the region.right: {:?}",
                            value
                        ))
                    }
                };
                let region_top = match stylebox.region.top {
                    Val::Percent(percent) => size.y * percent * 0.01,
                    Val::Px(px) => px,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the region.top: {:?}",
                            value
                        ))
                    }
                };
                let region_bottom = match stylebox.region.bottom {
                    Val::Percent(percent) => size.y * percent * 0.01,
                    Val::Px(px) => px,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the region.bottom: {:?}",
                            value
                        ))
                    }
                };
                let region = Rect {
                    min: Vec2::new(region_left, region_top),
                    max: Vec2::new(
                        (size.x - region_right).max(region_left),
                        (size.y - region_bottom).max(region_top),
                    ),
                };
                let size = region.size();

                let left = match stylebox.slice.left {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.x,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the slice.left: {:?}",
                            value
                        ))
                    }
                };
                let right = match stylebox.slice.right {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.x,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the slice.right: {:?}",
                            value
                        ))
                    }
                };
                let top = match stylebox.slice.top {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.y,
                    value => {
                        return cleanup(format!("Unsupported value for the slice.top: {:?}", value))
                    }
                };
                let bottom = match stylebox.slice.bottom {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / size.y,
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the slice.bottom: {:?}",
                            value
                        ))
                    }
                };
                let slice = UiRectF32::new(left, right, top, bottom);

                let left = match stylebox.width.left {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.x * slice.left),
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the width.left: {:?}",
                            value
                        ))
                    }
                };
                let right = match stylebox.width.right {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.x * slice.right),
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the width.right: {:?}",
                            value
                        ))
                    }
                };
                let top = match stylebox.width.top {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.y * slice.top),
                    value => {
                        return cleanup(format!("Unsupported value for the width.top: {:?}", value))
                    }
                };
                let bottom = match stylebox.width.bottom {
                    Val::Percent(percent) => percent * 0.01,
                    Val::Px(px) => px / (size.y * slice.bottom),
                    value => {
                        return cleanup(format!(
                            "Unsupported value for the width.bottom: {:?}",
                            value
                        ))
                    }
                };
                let width = UiRectF32::new(left, right, top, bottom);

                if let Some(mut computed) = computed {
                    computed.region = region;
                    computed.slice = slice;
                    computed.width = width;
                } else {
                    commands
                        .entity(entity)
                        .insert(StyleboxSlices::default())
                        .insert(ComputedStylebox {
                            region,
                            slice,
                            width,
                        });
                }
            }
        }
    }
}

struct StyleboxSlice {
    transform: Mat4,
    region: Rect,
}

#[derive(Component, Default)]
/// Holds transform for each slice
pub struct StyleboxSlices {
    items: Vec<StyleboxSlice>,
}
/// Calculates transforms for each slice based on `Node.size()` and `ComputedStylebox`
pub fn compute_stylebox_slices(
    mut query: Query<
        (&mut StyleboxSlices, &Node, &ComputedStylebox),
        Or<(Changed<Node>, Changed<ComputedStylebox>)>,
    >,
) {
    for (mut transforms, uinode, stylebox) in query.iter_mut() {
        if uinode.size() == Vec2::ZERO {
            continue;
        }
        transforms.items.clear();
        let size = uinode.size();
        let region = stylebox.region;
        let rpos = region.min;
        let rsize = region.size();

        let left = stylebox.slice.left;
        let right = stylebox.slice.right;
        let top = stylebox.slice.top;
        let bot = stylebox.slice.bottom;

        // compute part sizes in uinode space
        let w0 = left * rsize.x * stylebox.width.left;
        let w2 = right * rsize.x * stylebox.width.right;
        let w1 = size.x - w0 - w2;
        let h0 = top * rsize.y * stylebox.width.top;
        let h2 = bot * rsize.y * stylebox.width.bottom;
        let h1 = size.y - h0 - h2;

        let ui_x = &[0., w0, w0 + w1];
        let ui_y = &[0., h0, h0 + h1];
        let ui_width = &[w0, w1, w2];
        let ui_height = &[h0, h1, h2];

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
                let mut tr = Mat4::IDENTITY;
                tr *= Mat4::from_translation(offset.extend(0.));
                tr *= Mat4::from_scale(scale.extend(1.));
                transforms.items.push(StyleboxSlice {
                    transform: tr,
                    region: imgrect,
                });
            }
        }
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
pub fn extract_stylebox(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    ui_stack: Extract<Res<UiStack>>,
    windows: Extract<Res<Windows>>,
    images: Extract<Res<Assets<Image>>>,
    uinode_query: Extract<
        Query<(
            &Node,
            &GlobalTransform,
            &Stylebox,
            &StyleboxSlices,
            &ComputedVisibility,
            Option<&CalculatedClip>,
        )>,
    >,
) {
    let scale_factor = windows.scale_factor(WindowId::primary()) as f32;
    for (stack_index, entity) in ui_stack.uinodes.iter().enumerate() {
        let Ok((uinode, transform, stylebox, slices, visibility, clip)) = uinode_query.get(*entity) else {
            continue
        };
        if !visibility.is_visible() {
            continue;
        }
        let image = stylebox.texture.clone_weak();
        // Skip unloaded images
        if !images.contains(&image) {
            continue;
        }

        if uinode.size() == Vec2::ZERO {
            continue;
        }

        let img = images.get(&image).unwrap();
        let tr = transform.compute_matrix();

        for patch in slices.items.iter() {
            extracted_uinodes.uinodes.push(ExtractedUiNode {
                transform: tr * patch.transform,
                background_color: stylebox.modulate,
                rect: patch.region,
                image: image.clone_weak(),
                atlas_size: Some(img.size()),
                clip: clip.map(|clip| clip.clip),
                scale_factor,
                stack_index,
            });
        }
    }
}
