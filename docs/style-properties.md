<!-- THIS DOC IS GENERATED FROM RUST DOCSTRINGS -->
<!-- DO NOT EDIT IT BY HAND!!! -->
# Reference
| property | type | default |
|----------|------|---------|
|[`background-color`](#property-background-color)|[`$color`](#$color)|`transparent`|
|[`z-index`](#property-z-index)|`auto`**&#124;**[`$local`](#$local)**&#124;**[`$global`](#$global)|`auto`|
|[`align-content`](#property-align-content)|[`$ident`](#$ident)|`stretch`|
|[`align-items`](#property-align-items)|[`$ident`](#$ident)|`stretch`|
|[`flex-direction`](#property-flex-direction)|[`$ident`](#$ident)|`row`|
|[`flex-wrap`](#property-flex-wrap)|[`$ident`](#$ident)|`no-wrap`|
|[`justify-content`](#property-justify-content)|[`$ident`](#$ident)|`flex-start`|
|[`align-self`](#property-align-self)|[`$ident`](#$ident)|`auto`|
|[`flex-basis`](#property-flex-basis)|[`$val`](#$val)|`auto`|
|[`flex-grow`](#property-flex-grow)|[`$num`](#$num)|`0.0`|
|[`flex-shrink`](#property-flex-shrink)|[`$num`](#$num)|`1.0`|
|[`display`](#property-display)|[`$ident`](#$ident)|`flex`|
|[`overflow`](#property-overflow)|[`$ident`](#$ident)|`visible`|
|[`position`](#property-position)|[`$rect`](#$rect)|`-`|
|[`position-type`](#property-position-type)|[`$ident`](#$ident)|`relative`|
|[`bottom`](#property-bottom)|[`$val`](#$val)|`undefined`|
|[`left`](#property-left)|[`$val`](#$val)|`undefined`|
|[`right`](#property-right)|[`$val`](#$val)|`undefined`|
|[`top`](#property-top)|[`$val`](#$val)|`undefined`|
|[`aspect-ratio`](#property-aspect-ratio)|`none`**&#124;**[`$num`](#$num)|`none`|
|[`height`](#property-height)|[`$val`](#$val)|`undefined`|
|[`max-height`](#property-max-height)|[`$val`](#$val)|`undefined`|
|[`max-width`](#property-max-width)|[`$val`](#$val)|`undefined`|
|[`min-height`](#property-min-height)|[`$val`](#$val)|`undefined`|
|[`min-width`](#property-min-width)|[`$val`](#$val)|`undefined`|
|[`width`](#property-width)|[`$val`](#$val)|`undefined`|
|[`border-width`](#property-border-width)|[`$rect`](#$rect)|`-`|
|[`border-width-bottom`](#property-border-width-bottom)|[`$val`](#$val)|`undefined`|
|[`border-width-left`](#property-border-width-left)|[`$val`](#$val)|`undefined`|
|[`border-width-right`](#property-border-width-right)|[`$val`](#$val)|`undefined`|
|[`border-width-top`](#property-border-width-top)|[`$val`](#$val)|`undefined`|
|[`margin`](#property-margin)|[`$rect`](#$rect)|`-`|
|[`margin-bottom`](#property-margin-bottom)|[`$val`](#$val)|`undefined`|
|[`margin-left`](#property-margin-left)|[`$val`](#$val)|`undefined`|
|[`margin-right`](#property-margin-right)|[`$val`](#$val)|`undefined`|
|[`margin-top`](#property-margin-top)|[`$val`](#$val)|`undefined`|
|[`padding`](#property-padding)|[`$rect`](#$rect)|`-`|
|[`padding-bottom`](#property-padding-bottom)|[`$val`](#$val)|`undefined`|
|[`padding-left`](#property-padding-left)|[`$val`](#$val)|`undefined`|
|[`padding-right`](#property-padding-right)|[`$val`](#$val)|`undefined`|
|[`padding-top`](#property-padding-top)|[`$val`](#$val)|`undefined`|
|[`stylebox`](#property-stylebox)|`source, slice, region, width, modulate`|`-`|
|[`stylebox-modulate`](#property-stylebox-modulate)|[`$color`](#$color)|`white`|
|[`stylebox-region`](#property-stylebox-region)|[`$rect`](#$rect)|`0px`|
|[`stylebox-slice`](#property-stylebox-slice)|[`$rect`](#$rect)|`50%`|
|[`stylebox-source`](#property-stylebox-source)|`none`**&#124;**[`$string`](#$string)|`none`|
|[`stylebox-width`](#property-stylebox-width)|[`$rect`](#$rect)|`100%`|
|[`color`](#property-color)|[`$color`](#$color)|`#cfcfcf`|
|[`font`](#property-font)|`regular`**&#124;**`bold`**&#124;**`italic`**&#124;**`bold-italic`**&#124;**[`$string`](#$string)|`regular`|
|[`font-size`](#property-font-size)|[`$num`](#$num)|`24`|
# Types

## <a name="$color"></a>`$color`

<!-- @property-type=$color -->
Describes the `Color` value. Accepts color names (`white`, `red`) 
or hex codes (`#3fde1a`). List of predefined colors can be found
here (coming soon).
<!-- TODO: add link to color list -->
## <a name="$ident"></a>`$ident`

<!-- @property-type=$ident -->
Custom identifier: `no-wrap`, `none`, `auto`, etc. Each property accepts its own set 
of identifiers and describes them in the docs.
## <a name="$num"></a>`$num`

<!-- @property-type=$num -->
Numeric literal:
```css
flex-grow: 2.0
```
## <a name="$rect"></a>`$rect`

<!-- @property-type=$rect -->
Shorthand for describing `bevy::prelude::UiRect` using single line. Accepts 1 to 4
[`$val`](#$val) items related to edges of a box, like `margin` or `padding`.
- 1 value: specifies all edges: `margin: 10px`
- 2 values: the first value specifies vertical edges (top & bottom), the second
  value specifies horisontal edges (left & right): `padding: 5px 30%`
- 3 values: the first value specifies the top edge, the second specifies horisontal
  edges (left & right), the last one specifies the bottom edge: `border: 2px auto 5px`
- 4 values specifies all edges in top, right, bottom, left order (clock-wise):
  `margin: 5px 4px 3% auto`
 
## <a name="$string"></a>`$string`

<!-- @property-type=$string -->
String literal in double quotes:
```css
stylebox-source: "images/stylebox.png"
```
## <a name="$val"></a>`$val`

<!-- @property-type=$val -->
Size type representing `bevy::prelude::Val` type. Possible values:
- `auto` for `Val::Auto`
- `undefined` for `Val::Px(0.)`
- `px` suffixed for `Val::Px` (`25px`)
- `%` suffixed for `Val::Percent` (`25%`)
# Properties
## General
### <a name="property-background-color"></a>`background-color`
type: [`$color`](#$color)

default: `transparent`

TODO: write BacgroundColor description
<!-- @property-category=General -->
<!-- @property-name=background-color -->
<!-- @property-default=transparent -->
### <a name="property-z-index"></a>`z-index`
type: `auto`**|**[`$local`](#$local)**|**[`$global`](#$global)

default: `auto`

TODO: write ZIndex description
<!-- @property-category=General -->
<!-- @property-name=z-index -->
<!-- @property-default=auto -->
## Flex Container
### <a name="property-align-content"></a>`align-content`
type: [`$ident`](#$ident)

default: `stretch`

TODO: write AlignContent description
<!-- @property-category=Flex Container -->
<!-- @property-name=align-content -->
<!-- @property-default=stretch -->
### <a name="property-align-items"></a>`align-items`
type: [`$ident`](#$ident)

default: `stretch`

Specify element items alignment by providing value to `Style.align_items`:
```css
align-items: center;
```
 
The `align-items` property sets the `align-self` value on all direct children
as a group. In [Flexbox](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Flexible_Box_Layout).
it controls the alignment of items on the [Cross Axis](https://developer.mozilla.org/en-US/docs/Glossary/Cross_Axis).
 
Supported values:
- `flex-start`: The cross-start margin edges of the flex items are flushed with
  the cross-start edge of the line.
- `flex-end`: The cross-start margin edges of the flex items are flushed with
  the cross-start edge of the line.
- `center`: The flex items' margin boxes are centered within the line on the
  cross-axis. If the cross-size of an item is larger than the flex container,
  it will overflow equally in both directions.
- `baseline`: All flex items are aligned such that their
  [flex container baselines](https://drafts.csswg.org/css-flexbox-1/#flex-baselines) align
- `stretch`: Flex items are stretched such that the cross-size of the item's margin
  box is the same as the line while respecting width and height constraints.
<!-- @property-category=Flex Container -->
<!-- @property-name=align-items -->
<!-- @property-default=stretch -->
### <a name="property-flex-direction"></a>`flex-direction`
type: [`$ident`](#$ident)

default: `row`

Specify element flex direction by providing value to `Style.direction`:
```css
flex-direction: column;
```
 
The `flex-direction` property sets how flex items are placed in the flex
container defining the main axis and the direction (normal or reversed).
 
Supported values:
- `row`: The flex container's main-axis is defined to be the same as the
  text direction.
- `column`: The flex container's main-axies is defined to be vertical, items
  are placed from top to bottom.
- `row-reverse`: Behaves the same as `row` but opposite to the content direction.
- `column-reverse`: Behaves the same as `row` but items are placed from bottom to
   top.
<!-- @property-category=Flex Container -->
<!-- @property-name=flex-direction -->
<!-- @property-default=row -->
### <a name="property-flex-wrap"></a>`flex-wrap`
type: [`$ident`](#$ident)

default: `no-wrap`

Specify element flex wrap by providing value to `Style.flex_wrap`:
```css
flex-wrap: wrap;
```
 
The `flex-wrap` property sets whether flex items are forced onto one
line or can wrap onto multiple lines. If wrapping is allowed, it sets
the direction that lines are stacked.
 
Supported values:
- `no-wrap`: The flex items are laid out in a single line which may cause
  the flex container to overflow.
- `wrap`: The flex items break into multiple lines.
- `wrap-reverse`: Behaves the same as wrap but the new line is placed before
   the previous
<!-- @property-category=Flex Container -->
<!-- @property-name=flex-wrap -->
<!-- @property-default=no-wrap -->
### <a name="property-justify-content"></a>`justify-content`
type: [`$ident`](#$ident)

default: `flex-start`

TODO: write JustifyContent description
<!-- @property-category=Flex Container -->
<!-- @property-name=justify-content -->
<!-- @property-default=flex-start -->
## Flex Item
### <a name="property-align-self"></a>`align-self`
type: [`$ident`](#$ident)

default: `auto`

TODO: write AlignSelf description
<!-- @property-category=Flex Item -->
<!-- @property-name=align-self -->
<!-- @property-default=auto -->
### <a name="property-flex-basis"></a>`flex-basis`
type: [`$val`](#$val)

default: `auto`

Specify element flex basis by providing value to `Style.flex_basis`
using `val` syntax:
```css
flex-basis: 5px;
```
 
The `flex-basis` specifies the initial size of the flex item, before
any available space is distributed according to the flex factors.
<!-- (TODO: link val) -->
<!-- @property-category=Flex Item -->
<!-- @property-name=flex-basis -->
<!-- @property-default=auto -->
### <a name="property-flex-grow"></a>`flex-grow`
type: [`$num`](#$num)

default: `0.0`

Specify element flex grow by providing value to `Style.flex_grow`:
```css
flex-grow: 2.0;
```
 
The `flex-grow` defines the ability for a flex item to grow if necessary.
It accepts a unitless value that serves as a proportion. It dictates what
amount of the available space inside the flex container the item should
take up.
<!-- @property-category=Flex Item -->
<!-- @property-name=flex-grow -->
<!-- @property-default=0.0 -->
### <a name="property-flex-shrink"></a>`flex-shrink`
type: [`$num`](#$num)

default: `1.0`

Specify element flex shrink by providing value to `Style.flex_shrink`:
```css
flex-shrink: 3.0;
```
 
The flex-shrink property specifies how the item will shrink relative to
the rest of the flexible items inside the same container.
<!-- @property-category=Flex Item -->
<!-- @property-name=flex-shrink -->
<!-- @property-default=1.0 -->
## Layout Control
### <a name="property-display"></a>`display`
type: [`$ident`](#$ident)

default: `flex`

Specify element display by providing value to `Style.display`:
```css
display: none;
```
 
Supported values:
- `none`: turns off the display of an element so that it has no effect
  on layout (the document is rendered as though the element did not
  exist). All descendant elements also have their display turned off.
  To have an element take up the space that it would normally take, but
  without actually rendering anything
- `flex`: display element according to the
  [Flexbox](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Flexible_Box_Layout).
 
<!-- @property-category=Layout Control -->
<!-- @property-name=display -->
<!-- @property-default=flex -->
### <a name="property-overflow"></a>`overflow`
type: [`$ident`](#$ident)

default: `visible`

TODO: add OverflowProperty descripion
<!-- @property-category=Layout Control -->
<!-- @property-name=overflow -->
<!-- @property-default=visible -->
### <a name="property-position"></a>`position`
type: [`$rect`](#$rect)

Specify element position by providing values to `Style.position`:
```css
position: 2px 20% 10px auto;
```
<!-- @property-type=$rect -->
<!-- @property-category=Layout Control -->
<!-- @property-name=position -->
### <a name="property-position-type"></a>`position-type`
type: [`$ident`](#$ident)

default: `relative`

Specify how an element is positioned in a document acording to the `top`,
`right`, `bottom`, and `left` by providing value to `Style.position_type`:
```css
position-type: absolute;
```
 
Supported values:
- `absolute`: the element is removed from the normal document flow, and no
  space is created for the element in the page layout. It is positioned relative
  to its closest positioned ancestor. Its final position is determined by the
  values of `top`, `right`, `bottom`, and `left`.
- `relative`: the element is positioned according to the normal flow of the document
  and then offset *relative* to itself based on the values of `top`, `right`, `bottom`
  and left. The offset does not affect the position of any other elements.
<!-- @property-category=Layout Control -->
<!-- @property-name=position-type -->
<!-- @property-default=relative -->
## Layout Control Positioning
### <a name="property-bottom"></a>`bottom`
type: [`$val`](#$val)

default: `undefined`

Specify element bottom position by providing value to `Style.position.bottom`:
```css
bottom: 5px;
```
<!-- @property-category=Layout Control Positioning -->
<!-- @property-name=bottom -->
<!-- @property-default=undefined -->
### <a name="property-left"></a>`left`
type: [`$val`](#$val)

default: `undefined`

Specify element left position by providing value to `Style.position.left`:
```css
left: 5px;
```
<!-- @property-category=Layout Control Positioning -->
<!-- @property-name=left -->
<!-- @property-default=undefined -->
### <a name="property-right"></a>`right`
type: [`$val`](#$val)

default: `undefined`

Specify element right position by providing value to `Style.position.right`:
```css
right: 5px;
```
<!-- @property-category=Layout Control Positioning -->
<!-- @property-name=right -->
<!-- @property-default=undefined -->
### <a name="property-top"></a>`top`
type: [`$val`](#$val)

default: `undefined`

Specify element top position by providing value to `Style.position.top`:
```css
top: 5px;
```
<!-- @property-category=Layout Control Positioning -->
<!-- @property-name=top -->
<!-- @property-default=undefined -->
## Size Constraints
### <a name="property-aspect-ratio"></a>`aspect-ratio`
type: `none`**|**[`$num`](#$num)

default: `none`

Specify element preferred aspect ratio by providing value to
`Style.aspect_ratio`:
```css
aspect-ratio: 2.0;
```
 
The `aspect-ratio` property sets a preferred aspect ratio for
the box, which will be used in the calculation of auto sizes
and some other layout functions.
<!-- @property-category=Size Constraints -->
<!-- @property-name=aspect-ratio -->
<!-- @property-default=none -->
### <a name="property-height"></a>`height`
type: [`$val`](#$val)

default: `undefined`

Specify element preferred height by providing value to `Style.size.height`:
```css
height: 5px;
```
<!-- @property-category=Size Constraints -->
<!-- @property-name=height -->
<!-- @property-default=undefined -->
### <a name="property-max-height"></a>`max-height`
type: [`$val`](#$val)

default: `undefined`

Specify element maximum height by providing value to `Style.max_size.height`:
```css
max-height: 5px;
```
<!-- @property-category=Size Constraints -->
<!-- @property-name=max-height -->
<!-- @property-default=undefined -->
### <a name="property-max-width"></a>`max-width`
type: [`$val`](#$val)

default: `undefined`

Specify element maximum width by providing value to `Style.max_size.width`:
```css
max-width: 5px;
```
<!-- @property-category=Size Constraints -->
<!-- @property-name=max-width -->
<!-- @property-default=undefined -->
### <a name="property-min-height"></a>`min-height`
type: [`$val`](#$val)

default: `undefined`

Specify element minimum height by providing value to `Style.min_size.height`:
```css
min-height: 5px;
```
<!-- @property-category=Size Constraints -->
<!-- @property-name=min-height -->
<!-- @property-default=undefined -->
### <a name="property-min-width"></a>`min-width`
type: [`$val`](#$val)

default: `undefined`

Specify element minimum width by providing value to `Style.min_size.width`:
```css
min-width: 5px;
```
<!-- @property-category=Size Constraints -->
<!-- @property-name=min-width -->
<!-- @property-default=undefined -->
### <a name="property-width"></a>`width`
type: [`$val`](#$val)

default: `undefined`

Specify element preferred width by providing value to `Style.size.width`:
```css
width: 5px;
```
<!-- @property-category=Size Constraints -->
<!-- @property-name=width -->
<!-- @property-default=undefined -->
## Spacing
### <a name="property-border-width"></a>`border-width`
type: [`$rect`](#$rect)

Specify element border width by providing values to `Style.border`:
```css
border-width: 2px 20% 10px auto;
```
 
The `border-width` property specifies the width of the four borders.
<!-- @property-type=$rect -->
<!-- @property-category=Spacing -->
<!-- @property-name=border-width -->
### <a name="property-border-width-bottom"></a>`border-width-bottom`
type: [`$val`](#$val)

default: `undefined`

Specify element bottom border width by providing value to `Style.border.bottom`:
```css
border-width-bottom: 5px;
```
<!-- @property-category=Spacing -->
<!-- @property-name=border-width-bottom -->
<!-- @property-default=undefined -->
### <a name="property-border-width-left"></a>`border-width-left`
type: [`$val`](#$val)

default: `undefined`

Specify element left border width by providing value to `Style.border.left`:
```css
border-width-left: 5px;
```
<!-- @property-category=Spacing -->
<!-- @property-name=border-width-left -->
<!-- @property-default=undefined -->
### <a name="property-border-width-right"></a>`border-width-right`
type: [`$val`](#$val)

default: `undefined`

Specify element right border width by providing value to `Style.border.right`:
```css
border-width-right: 5px;
```
<!-- (TODO: link val) -->
<!-- @property-category=Spacing -->
<!-- @property-name=border-width-right -->
<!-- @property-default=undefined -->
### <a name="property-border-width-top"></a>`border-width-top`
type: [`$val`](#$val)

default: `undefined`

Specify element top border width by providing value to `Style.border.top`:
```css
border-width-top: 5px;
```
<!-- @property-category=Spacing -->
<!-- @property-name=border-width-top -->
<!-- @property-default=undefined -->
### <a name="property-margin"></a>`margin`
type: [`$rect`](#$rect)

Specify element margin by providing values to `Style.margin`:
```css
margin: 2px 20% 10px auto;
```
 
Margins are used to create space around elements, outside of
any defined borders.
<!-- @property-type=$rect -->
<!-- @property-category=Spacing -->
<!-- @property-name=margin -->
### <a name="property-margin-bottom"></a>`margin-bottom`
type: [`$val`](#$val)

default: `undefined`

Specify element bottom margin by providing value to `Style.margin.bottom`:
```css
margin-bottom: 5px;
```
 
Margins are used to create space around elements, outside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=margin-bottom -->
<!-- @property-default=undefined -->
### <a name="property-margin-left"></a>`margin-left`
type: [`$val`](#$val)

default: `undefined`

Specify element left margin by providing value to `Style.margin.left`:
```css
margin-left: 5px;
```
 
Margins are used to create space around elements, outside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=margin-left -->
<!-- @property-default=undefined -->
### <a name="property-margin-right"></a>`margin-right`
type: [`$val`](#$val)

default: `undefined`

Specify element right margin by providing value to `Style.margin.right`:
```css
margin-right: 5px;
```
 
Margins are used to create space around elements, outside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=margin-right -->
<!-- @property-default=undefined -->
### <a name="property-margin-top"></a>`margin-top`
type: [`$val`](#$val)

default: `undefined`

Specify element top margin by providing value to `Style.margin.top`:
```css
margin-top: 5px;
```
 
Margins are used to create space around elements, outside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=margin-top -->
<!-- @property-default=undefined -->
### <a name="property-padding"></a>`padding`
type: [`$rect`](#$rect)

Specify element padding by providing values to `Style.padding`:
```css
padding: 2px 20% 10px auto;
```
 
Padding is used to create space around an element's content, inside of
any defined borders.
<!-- @property-type=$rect -->
<!-- @property-category=Spacing -->
<!-- @property-name=padding -->
### <a name="property-padding-bottom"></a>`padding-bottom`
type: [`$val`](#$val)

default: `undefined`

Specify element bottom padding by providing value to `Style.padding.bottom`:
```css
padding-bottom: 5px;
```
 
Padding is used to create space around an element's content, inside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=padding-bottom -->
<!-- @property-default=undefined -->
### <a name="property-padding-left"></a>`padding-left`
type: [`$val`](#$val)

default: `undefined`

Specify element left padding by providing value to `Style.padding.left`:
```css
padding-left: 5px;
```
 
Padding is used to create space around an element's content, inside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=padding-left -->
<!-- @property-default=undefined -->
### <a name="property-padding-right"></a>`padding-right`
type: [`$val`](#$val)

default: `undefined`

Specify element right padding by providing value to `Style.padding.right`:
```css
padding-right: 5px;
```
 
Padding is used to create space around an element's content, inside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=padding-right -->
<!-- @property-default=undefined -->
### <a name="property-padding-top"></a>`padding-top`
type: [`$val`](#$val)

default: `undefined`

Specify element top padding by providing value to `Style.padding.top`:
```css
padding-top: 5px;
```
 
Padding is used to create space around an element's content, inside of
any defined borders.
<!-- @property-category=Spacing -->
<!-- @property-name=padding-top -->
<!-- @property-default=undefined -->
## Stylebox
### <a name="property-stylebox"></a>`stylebox`
type: `source, slice, region, width, modulate`

Specify how to fill the element with region of image sliced by 9 parts.
The `stylebox` property is shorthand property for:
- `stylebox-source` specifies the source of the image
- `stylebox-slice` specifies how to slice the image
- `stylebox-region` specifies the region of the image
- `stylebox-width` specifies how to resize edges
- `stylebox-modulate` specifies what color the image should be multiplied by
 
The format of property is:
```
source, slice, width, region, modulate
```
Every tail element is optional (you can omit `modulate` for example. If you do,
you can ompit `region` then. And so on.)
 
Example:
```css
  stylebox: "background.png", 16px 12px, 100%, 0px, blue
  stylebox: "background.png", 5px 20%
```
<!-- @property-type=source, slice, region, width, modulate -->
<!-- @property-category=Stylebox -->
<!-- @property-name=stylebox -->
### <a name="property-stylebox-modulate"></a>`stylebox-modulate`
type: [`$color`](#$color)

default: `white`

The `stylebox-modulate` property specifies what color the original image
should be multiplied by.
<!-- @property-category=Stylebox -->
<!-- @property-name=stylebox-modulate -->
<!-- @property-default=white -->
### <a name="property-stylebox-region"></a>`stylebox-region`
type: [`$rect`](#$rect)

default: `0px`

The `stylebox-region` property specifies which region of the image should be sliced.
By default the hole area of image defined by `stylebox-source` is used.
Property accepts [`$rect`](#$rect):
- `px` values defines exact offset from the edges in pixels
- `%` values defines offset from the edges relative to the image size
- `auto` & `undefined` treated as `0px`
<!-- @property-category=Stylebox -->
<!-- @property-name=stylebox-region -->
<!-- @property-default=0px -->
### <a name="property-stylebox-slice"></a>`stylebox-slice`
type: [`$rect`](#$rect)

default: `50%`

The `stylebox-slice` property specifies how to slice the image region
specified by `stylebox-source` and `stylebox-region`. The image is
always sliced into nine sections: four corners, four edges and the middle.
Property accepts [`$rect`](#$rect):
- when `px` specified, region sliced to the exact amount of pixels
- when `%` specified, region sliced relative to it size
- `auto` & `undefined` treated as `50%`
<!-- @property-category=Stylebox -->
<!-- @property-name=stylebox-slice -->
<!-- @property-default=50% -->
### <a name="property-stylebox-source"></a>`stylebox-source`
type: `none`**|**[`$string`](#$string)

default: `none`

The `stylebox-source` property specifies the path to the image to be used
as a stylebox. The property accepts `String` values.
<!-- @property-category=Stylebox -->
<!-- @property-name=stylebox-source -->
<!-- @property-default=none -->
### <a name="property-stylebox-width"></a>`stylebox-width`
type: [`$rect`](#$rect)

default: `100%`

The `stylebox-width` property specifies the width of the edgets of sliced region.
Property accepts [`$rect`](#$rect):
- edges specified by `px` values resizes to exact amout of pixels
- edges specified by `%` resized relative to width provided by `stylebox-slice`
- `auto` & `undefined` treated as `100%`
<!-- @property-category=Stylebox -->
<!-- @property-name=stylebox-width -->
<!-- @property-default=100% -->
## Text
### <a name="property-color"></a>`color`
type: [`$color`](#$color)

default: `#cfcfcf`

TODO: remove depricate ColorProperty
<!-- @property-category=Text -->
<!-- @property-name=color -->
<!-- @property-default=#cfcfcf -->
### <a name="property-font"></a>`font`
type: `regular`**|**`bold`**|**`italic`**|**`bold-italic`**|**[`$string`](#$string)

default: `regular`

TODO: wtite FontProperty description
<!-- @property-category=Text -->
<!-- @property-name=font -->
<!-- @property-default=regular -->
### <a name="property-font-size"></a>`font-size`
type: [`$num`](#$num)

default: `24`

TODO: write FontSizeProperty description
<!-- @property-category=Text -->
<!-- @property-name=font-size -->
<!-- @property-default=24 -->
