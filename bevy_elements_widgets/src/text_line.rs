use bevy::{prelude::*, ecs::query::QueryItem};
use bevy_elements_core::{*, property::PropertyValues};

#[derive(Default)]
pub struct TextLinePlugin;

impl Plugin for TextLinePlugin {
    fn build(&self, app: &mut App) {
        app.register_property::<TextLineFontColorProperty>();
        app.register_property::<TextLineFontProperty>();
        app.register_property::<TextLineFontSizeProperty>();
        app.add_system(update_text_line);
    }
}

#[derive(Component)]
pub struct TextLine {
    pub style: TextStyle,
    pub value: String
}

// pub struct TextLineLayout

widget!(TextLine,
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
    default_font: Res<DefaultFont>
=> {
    let text = bindattr!(ctx, commands, value:String => Self.value);
    let style = TextStyle {
        font: default_font.0.clone(),
        font_size: 24.0,
        color: Color::WHITE,
    };
    let line = Self {
        style: style.clone(),
        value: text.unwrap_or("".to_string())
    };
    commands
        .entity(ctx.element)
        .insert(line)
        .insert(ManualTextProperties)
        .insert(TextBundle::from_section(
            "".to_string(),
            TextStyle {
                font: default_font.0.clone(),
                font_size: 24.0,
                color: Color::WHITE,
            },
        ));
});

fn update_text_line(
    mut lines: Query<(&TextLine, &mut Text, &mut Style), Changed<TextLine>>
) {
    for (line, mut text, mut style) in lines.iter_mut() {
        style.size.height = Val::Px(line.style.font_size);
        text.sections[0].value = line.value.clone();
        text.sections[0].style = line.style.clone();
    }
}

#[derive(Default)]
pub(crate) struct TextLineFontColorProperty;

impl Property for TextLineFontColorProperty {
    type Cache = Color;
    type Components = &'static mut TextLine;
    type Filters = With<Node>;

    fn name() -> Tag {
        tag!("color")
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, ElementsError> {
        if let Some(color) = values.color() {
            Ok(color)
        } else {
            Err(ElementsError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        mut components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.style.color = *cache;
    }
}

#[derive(Default)]
pub(crate) struct TextLineFontProperty;

impl Property for TextLineFontProperty {
    type Cache = String;
    type Components = &'static mut TextLine;
    type Filters = With<Node>;

    fn name() -> Tag {
        tag!("font")
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, ElementsError> {
        if let Some(path) = values.string() {
            Ok(path)
        } else {
            Err(ElementsError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        mut components: QueryItem<Self::Components>,
        asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.style.font = asset_server.load(cache);
    }
}

#[derive(Default)]
pub(crate) struct TextLineFontSizeProperty;

impl Property for TextLineFontSizeProperty {
    type Cache = f32;
    type Components = &'static mut TextLine;
    type Filters = With<Node>;

    fn name() -> Tag {
        tag!("font-size")
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, ElementsError> {
        if let Some(size) = values.f32() {
            Ok(size)
        } else {
            Err(ElementsError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        mut components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.style.font_size = *cache;
    }
}