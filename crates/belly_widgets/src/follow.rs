use super::common::*;
use belly_core::build::*;
use belly_macro::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::Follow;
    pub use super::FollowWidgetExtension;
}

pub(crate) struct FollowPlugin;
impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<FollowWidget>();
        app.add_systems(Update, follow_system);
    }
}

#[derive(Component)]
pub struct Follow {
    target: Entity,
}

impl FromWorldAndParams for Follow {
    fn from_world_and_params(_: &mut World, params: &mut belly_core::eml::Params) -> Self {
        Follow {
            target: params
                .try_get("target")
                .expect("Missing required `target` param"),
        }
    }
}

#[widget]
#[param(target:Entity => Follow:target)]
fn follow(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.render(eml! {
        <span s:left=managed() s:top=managed() s:position-type="absolute">
            {content}
        </span>
    })
}

fn follow_system(
    mut follows: Query<(Entity, &Follow, &mut Style, &Node)>,
    transforms: Query<&GlobalTransform>,
    mut commands: Commands,
    windows: Query<&Window>,
) {
    for window in windows.iter() {
        for (entity, follow, mut style, node) in follows.iter_mut() {
            let Ok(tr) = transforms.get(follow.target) else {
                commands.entity(entity).despawn_recursive();
                continue;
            };
            let pos = tr.translation();
            style.left = Val::Px((pos.x + window.width() * 0.5 - 0.5 * node.size().x).round());
            style.top = Val::Px((window.height() * 0.5 - pos.y - 0.5 * node.size().y).round());
        }
    }
}
