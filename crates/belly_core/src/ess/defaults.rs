use crate::eml::build::ElementBuilderRegistry;
use crate::ess::PropertyExtractor;
use crate::ess::PropertyTransformer;
use crate::ess::StyleSheet;
use crate::ess::StyleSheetParser;
use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct Defaults {
    pub regular_font: Handle<Font>,
    pub italic_font: Handle<Font>,
    pub bold_font: Handle<Font>,
    pub bold_italic_font: Handle<Font>,
    pub style_sheet: Handle<StyleSheet>,
}

pub fn setup_defaults(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    mut defaults: ResMut<Defaults>,
    elements_registry: Res<ElementBuilderRegistry>,
    extractor: Res<PropertyExtractor>,
    validator: Res<PropertyTransformer>,
) {
    let font_bytes = include_bytes!("assets/Exo2-ExtraLight.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.regular_font = font_handle;
    let font_bytes = include_bytes!("assets/Exo2-ExtraLightItalic.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.italic_font = font_handle;
    let font_bytes = include_bytes!("assets/Exo2-SemiBold.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.bold_font = font_handle;
    let font_bytes = include_bytes!("assets/Exo2-SemiBoldItalic.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.bold_italic_font = font_handle;

    let parser = StyleSheetParser::new(validator.clone(), extractor.clone());
    let mut rules = parser.parse(
        r#"
            * {
                font: regular;
                color: #cfcfcf;
                font-size: 22px;
                display: flex;
                background-color: transparent;
            }
        "#,
    );
    for rule in elements_registry.styles(parser) {
        rules.push(rule)
    }
    commands.add(StyleSheet::add_default(rules));
}
