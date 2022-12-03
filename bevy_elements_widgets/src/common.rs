use bevy::prelude::*;
use bevy_elements_core::*;

pub struct CommonsPlugin;

impl Plugin for CommonsPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<Div>();
        app.register_widget::<Body>();
        app.register_widget::<Label>();
    }
}

pub trait CommonWidgetsExtension {
    fn div() -> ElementBuilder {
        Div::as_builder()
    }

    fn body() -> ElementBuilder {
        Body::as_builder()
    }

    fn label() -> ElementBuilder {
        Label::as_builder()
    }

    fn Label() -> ElementBuilder {
        Label::as_builder()
    }
}
impl CommonWidgetsExtension for Elements {}

#[derive(Component)]
pub(crate) struct Div;
impl Widget for Div {
    fn names() -> &'static [&'static str] {
        &["div"]
    }
}

impl WidgetBuilder for Div {
    fn construct(ctx: &mut ElementContext) {
        let content = ctx.content();
        ctx.insert(ElementBundle::default()).push_children(&content);
    }
}

#[derive(Component)]
pub(crate) struct Body;
impl Widget for Body {
    fn names() -> &'static [&'static str] {
        &["body"]
    }
}
impl WidgetBuilder for Body {
    fn construct(ctx: &mut ElementContext) {
        let content = ctx.content();
        ctx.insert(ElementBundle {
            node: NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    align_content: AlignContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(Interaction::None)
        .push_children(&content);
    }
}

#[derive(Component)]
pub struct Label {
    value: String,
}
impl Widget for Label {
    fn names() -> &'static [&'static str] {
        &["Label", "label"]
    }

    fn construct_component(world: &mut World) -> Option<Self> {
        Some(Label {
            value: "".to_string(),
        })
    }

    fn bind_component(&mut self, ctx: &mut ElementContext) {
        if let Some(value) = bindattr!(ctx, value:String => Self.value) {
            self.value = value;
        }
        let entity = ctx.entity();
        ctx.commands()
            .add(bind!(entity, Label.value => entity, Text.sections[0].value));
    }
}

impl WidgetBuilder for Label {
    fn setup(&mut self, ctx: &mut ElementContext) {
        ctx.insert(TextElementBundle::default());
    }
}
