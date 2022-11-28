use bevy::{prelude::*, ecs::query::QueryItem};
use bevy_elements_core::{*, property::PropertyValues};

#[derive(Default)]
pub struct TextLinePlugin;

impl Plugin for TextLinePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_text_line);
    }
}

#[derive(Component)]
pub struct TextLine {
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
        value: text.unwrap_or("".to_string())
    };
    commands
        .entity(ctx.element)
        .insert(line)
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
    mut lines: Query<(&TextLine, &mut Text, &mut Style), Or<(Changed<Text>, Changed<TextLine>)>>
) {
    for (line, mut text, mut style) in lines.iter_mut() {
        style.size.height = Val::Px(text.sections[0].style.font_size);
        text.sections[0].value = line.value.clone();
    }
}