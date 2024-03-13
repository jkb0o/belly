use crate::input::button::*;
use crate::range::*;
use belly_core::build::*;
use belly_core::input;
use belly_macro::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::SliderWidgetExtension;
}

pub(crate) struct SliderPlugin;
impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            handle_grabber_input.in_set(input::InputSystemsSet),
        );
        app.register_widget::<SliderWidget>();
    }
}

#[widget]
#[extends(RangeWidget)]
#[styles(
    slider .slider-grabber {
      margin: 0px;
      min-width: 16px;
      min-height: 16px;
      width: 16px;
      height: 16px;
    }
)]
fn slider(ctx: &mut WidgetContext) {
    let grabber = SliderGrabber {
        slider: ctx.entity(),
    };
    let params = ctx.params();
    ctx.render(eml! {
        <range c:slider params=params>
            <slot separator>
                <button with=grabber mode="instant" c:slider-grabber>
                </button>
            </slot>
        </range>
    })
}
#[derive(Component)]
struct SliderGrabber {
    slider: Entity,
}

fn handle_grabber_input(
    mut events: EventReader<PointerInput>,
    mut sliders: Query<&mut Range>,
    grabbers: Query<(Entity, &SliderGrabber, &Node)>,
    mut styles: Query<&mut Style>,
    holders: Query<(&GlobalTransform, &Node)>,

    mut active_grabber: Local<Option<Entity>>,
    mut active_slider: Local<Option<Entity>>,
) {
    for ev in events.read() {
        // info!("{ev:?}, {active_grabber:?}");
        // if active_grabber.is_some() && ev.drag_stop() {
        if ev.drag_start() && active_grabber.is_none() {
            let Some((entity, grabber, _)) = ev.entities.iter().find_map(|e| grabbers.get(*e).ok())
            else {
                return;
            };
            *active_grabber = Some(entity);
            *active_slider = Some(grabber.slider);
            // if let Ok((mut slider, _)) = sliders.get_mut(active_slider.unwrap()) {
            //     slider.sliding = true;
            // }
        } else if active_grabber.is_some() && (ev.dragging() || ev.drag_stop()) {
            let entity = active_grabber.unwrap();
            let Ok(mut range) = sliders.get_mut(active_slider.unwrap()) else {
                continue;
            };
            let Ok((_, _, gnode)) = grabbers.get(entity) else {
                continue;
            };
            let Ok((htr, holder_node)) = holders.get(range.holder) else {
                continue;
            };
            let Ok((_, high_node)) = holders.get(range.high_span) else {
                continue;
            };
            let Ok((_, low_node)) = holders.get(range.low_span) else {
                continue;
            };
            let Ok(mut style) = styles.get_mut(range.low_span) else {
                continue;
            };
            let grabber_offset = gnode.size() * 0.5;
            let pos = ev.pos - htr.translation().truncate() + holder_node.size() * 0.5;
            let mut offset = (pos - grabber_offset).min(holder_node.size() - gnode.size());
            offset.y = holder_node.size().y - offset.y - gnode.size().y;
            offset.y = offset.y.min(holder_node.size().y - gnode.size().y);
            let offset = offset.max(Vec2::ZERO);
            let relative = offset / (low_node.size() + high_node.size());
            match range.mode {
                LayoutMode::Horizontal => {
                    style.min_width = Val::Px(offset.x);
                    range.value.set_relative(relative.x);
                }
                LayoutMode::Vertical => {
                    style.min_height = Val::Px(offset.y);
                    range.value.set_relative(relative.y);
                }
            }
            if ev.drag_stop() {
                // slider.sliding = false;
                *active_slider = None;
                *active_grabber = None;
            }
        }
    }
}
