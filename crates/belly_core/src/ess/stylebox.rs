use crate::build::*;
use crate::ess::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_stylebox::*;

pub struct StyleboxPropertyPlugin;
impl Plugin for StyleboxPropertyPlugin {
    fn build(&self, app: &mut App) {
        app.register_compound_property::<StyleboxProperty>();
        app.register_property::<StyleboxSourceProperty>();
        app.register_property::<StyleboxModulateProperty>();
        app.register_property::<StyleboxRegionProperty>();
        app.register_property::<StyleboxSliceProperty>();
        app.register_property::<StyleboxWidthProperty>();
    }
}

compound_style_property! {
    #[doc = " Specify how to fill the element with sliced by 9 parts region of image."]
    #[doc = " The `stylebox` property is shorthand property for:"]
    #[doc = " - `stylebox-source` specifies the source of the image"]
    #[doc = " - `stylebox-region` specifies the region of the image"]
    #[doc = " - `stylebox-slice` specifies how to slice the image"]
    #[doc = " - `stylebox-width` specifies how to resize edges"]
    #[doc = " - `stylebox-modulate` specifies what color the image should be multiplied by"]
    #[doc = " "]
    #[doc = " The format of property is:"]
    #[doc = " "]
    #[doc = " 'source, slice, width, region, modulate'"]
    #[doc = "  "]
    #[doc = " Example:"]
    #[doc = " ```css"]
    #[doc = "   stylebox: \"background.png\", 16px 12px, 100%, 0px, blue"]
    #[doc = " ```"]
    StyleboxProperty("stylebox", value) {
        let props = match value {
            Variant::String(unparsed) => StyleProperty::try_from(unparsed)?,
            Variant::Style(prop) => prop,
            v => return Self::error(format!("Don't know how to extract stylebox from {v:?}"))
        };
        let mut stream = props.as_stream();
        let mut result = HashMap::default();
        if let Some(path) = stream.single() {
            result.insert(tag!("stylebox-source"), PropertyValue::new(path.string()?));
        }
        if let Some(slice) = stream.compound() {
            result.insert(tag!("stylebox-slice"), PropertyValue::new(slice.rect()?));
        }
        if let Some(width) = stream.compound() {
            result.insert(tag!("stylebox-width"), PropertyValue::new(width.rect()?));
        }
        if let Some(region) = stream.compound() {
            result.insert(tag!("stylebox-region"), PropertyValue::new(region.rect()?));
        }
        if let Some(modulate) = stream.single() {
            result.insert(tag!("stylebox-modulate"), PropertyValue::new(modulate.color()?));
        }
        Ok(result)
    }
}

style_property! {
    #[doc = " The `stylebox-source` property specifies the path to the image to be used"]
    #[doc = " as a stylebox. The property accepts `String` values."]
    StyleboxSourceProperty("stylebox-source") {
        Item = String;
        Components = &'static mut Stylebox;
        Filters = With<Node>;
        Parse = |v| v.string();
        Apply = |value, stylebox, assets, _commands, _entity| {
            let image = assets.load(value);
            if stylebox.texture != image {
                stylebox.texture = image;
            }
        };
    }
}

style_property! {
    #[doc = " The `stylebox-slice` property specifies how to slice the image region"]
    #[doc = " specified by `stylebox-source` and `stylebox-region`. The image is"]
    #[doc = " always sliced into nine sections: four corners, four edges and the middle."]
    #[doc = " The property accepts `rect-shorthand` values limited to `%` or `px` (no"]
    #[doc = " auto or undefined values are supported):"]
    #[doc = " - when `px` specified, region sliced to the exact amount of pixels"]
    #[doc = " - when `%` specified, region sliced relative to it size"]
    #[doc = " <!-- (TODO: link rect-shorthand) -->"]
    StyleboxSliceProperty("stylebox-slice") {
        Item = UiRect;
        Components = &'static mut Stylebox;
        Filters = With<Node>;
        Parse = |v| v.rect();
        Apply = |value, stylebox, _assets, _commands, _entity| {
            if stylebox.slice != *value {
                stylebox.slice = *value
            }
        };
    }
}

style_property! {
    #[doc = " The `stylebox-width` property specifies the width of the edgets of sliced region."]
    #[doc = " The property accepts `rect-shorthand` values limited to `%` or `px` (no"]
    #[doc = " auto or undefined values are supported):"]
    #[doc = " - edges specified by `px` values resizes to exact amout of pixels"]
    #[doc = " - edges specified by `%` resized relative to width provided by `stylebox-slice`"]
    #[doc = " "]
    #[doc = " Default value for `stylebox-width` is `100%`"]
    #[doc = " <!-- (TODO: link rect-shorthand) -->"]
    StyleboxWidthProperty("stylebox-width") {
        Item = UiRect;
        Components = &'static mut Stylebox;
        Filters = With<Node>;
        Parse = |v| v.rect();
        Apply = |value, stylebox, _assets, _commands, _entity| {
            if stylebox.width != *value {
                stylebox.width = *value
            }
        };
    }
}

style_property! {
    #[doc = " The `stylebox-region` property specifies which region of the image should be sliced."]
    #[doc = " By default the hole area of image defined by `stylebox-source` is used."]
    #[doc = " The property accepts `rect-shorthand` values limited to `%` or `px` (no"]
    #[doc = " auto or undefined values are supported)."]
    #[doc = " - `px` values defines exact offset from the edges in pixels"]
    #[doc = " - `%` values defines offset from the edges relative to the image size"]
    #[doc = " "]
    #[doc = " Default value for `stylebox-region` is `0px`"]
    #[doc = " <!-- (TODO: link rect-shorthand) -->"]
    StyleboxRegionProperty("stylebox-region") {
        Item = UiRect;
        Components = &'static mut Stylebox;
        Filters = With<Node>;
        Parse = |v| v.rect();
        Apply = |value, stylebox, _assets, _commands, _entity| {
            if stylebox.region != *value {
                stylebox.region = *value
            }
        };
    }
}

style_property! {
    #[doc = " The `stylebox-modulate` property specifies what color the original image"]
    #[doc = " should be multiplied by."]
    #[doc = " The property accepts `color` values (hex representation or color name)."]
    #[doc = " "]
    #[doc = " Default value for `stylebox-modulate` is `white`"]
    #[doc = " <!-- (TODO: link color) -->"]
    StyleboxModulateProperty("stylebox-modulate") {
        Item = Color;
        Components = &'static mut Stylebox;
        Filters = With<Node>;
        Parse = |v| v.color();
        Apply = |value, stylebox, _assets, _commands, _entity| {
            if stylebox.modulate != *value {
                stylebox.modulate = *value
            }
        };
    }
}
