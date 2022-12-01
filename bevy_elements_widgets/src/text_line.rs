use bevy::prelude::*;
use bevy_elements_core::*;

#[derive(Default)]
pub struct TextLinePlugin;

impl Plugin for TextLinePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_text_line);
    }
}

#[derive(Component)]
pub struct TextLine {
    pub value: String,
}

// pub struct TextLineLayout

widget!(TextLine,
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands
=> {
    let text = bindattr!(ctx, commands, value:String => Self.value);
    let line = Self {
        value: text.unwrap_or("".to_string())
    };
    commands
        .entity(ctx.element)
        .insert(line)
        .insert(TextBundle::from_section(
            "".to_string(),
            Default::default()
        ));

});

fn update_text_line(
    mut lines: Query<(&TextLine, &mut Text, &mut Style), Or<(Changed<Text>, Changed<TextLine>)>>,
) {
    for (line, mut text, mut style) in lines.iter_mut() {
        style.size.height = Val::Px(text.sections[0].style.font_size);
        text.sections[0].value = line.value.clone();
    }
}
