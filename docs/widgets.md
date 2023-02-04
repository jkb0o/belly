## body

The `<body>` tag defines a ui content (text, images, links, inputs, etc.).
It occupies the entire space of the window and should be treated as root
container for other elements.

> **_NOTE:_** in the future releases it is possible the only single `<body>`
> element on the page would be allowed.

<!-- widget-category: common -->


## br

The `<br/>` tag inserts single line break. `<br/>` height is
zero, so combining multiple `<br/>` tags has no effect. Use
[`<brl/>`](BrlWidget) if you want to insert extra empty line.


## brl

The `<brl/>` tag inserts line break **and** extra empty line
with the height of the current font-size. If you only need
to insert single line break use [`<br/>`](br) tag instead.


## button

The `<button>` tag defines a clickable button.
Inside a `<button>` element you can put text (and tags
like `<i>`, `<b>`, `<strong>`, `<br>`, `<img>`, etc.)
A button can emit `pressed` and `released` signals.
The button behaviour defined by the `mode` param.
When changing its pressed state, button adds `:pressed` ess
state to element if it pressed and remove `:pressed` if it releases.


Params:

- `pressed:` `bool`
if button is pressed or not
 
- `mode:` `BtnMode`
Specifies the button behavior:
  
  - `press`: When the button is clicked, it will act as if it was pressed
    for a single frame after the mouse/touch is released if it was down on
    the same button. The `released` signal will emit on the same frame,
    immediately following the `pressed` signal.
  
  - `instant`: The button will act as if it was pressed immediately after
    the mouse/touch is clicked, unless it is released first. The button will
    emit the `pressed` signal immediately after the mouse/touch is clicked,
    and the `released` signal after the mouse/touch is released.
  
  - `toggle`: The button will toggle its pressed state on and off. The
    `pressed` signal will emit when the button is pressed down and the
    `released` signal will emit when the button is released, unless it is
    still pressed down, in which case the `pressed` signal will not be
    emitted.
  
  - `repeat($speed)`: This mode is similar to `instant`, but the `pressed`
    signal will also emit periodically based on `$speed`. `$speed` can be
    a constant value or a sequence of delays between emissions. The following
    values are accepted:
    - `fast`, `normal`, and `slow` emit signals starting with some base delay
      and reduce it over time until the minimum delay is reached.
    - A sequence in the form `0.5 0.4 0.4 0.25`, with any number of elements,
      where each element specifies the delay between the previous `pressed`
      emission and the next one.
  
  - `group($name)`: Associates the button with a virtual named group. Buttons
    in the same group will act like toggle buttons, but only one button may
    have the pressed state at a time.
 
- `value:` `String`
Specifies the `<button>` value passed to parent `<buttongroup>` 
when this button becomes pressed.

## buttongroup

A container for multiple toggle buttons. When a button inside a
`<buttongroup>` is clicked, it will toggle its pressed state and emit the
`pressed` and `released` signals as appropriate. The `<buttongroup>` will
update its own `value` property to match the pressed state of the buttons.

When you set the `value` property of a `<buttongroup>`, the corresponding
button will become pressed, and all other buttons in the group will have
their pressed state removed.


Params:

- `value:` `String`
The current value of the `<buttongroup>`. When you set this property, the
corresponding button will become pressed, and all other buttons in the
group will have their pressed state removed.

## div

The `<div>` tag is an empty container that is used to define
a division or a section. It does not affect the content or layout
and is used to group `eml` elements to be styled with `ess`.


## img

The `<img>` is used to load image and show it content on the UI screen.


Params:

- `src:` `ImageSource`
Specifies the path to the image or custom `Handle<Image>`
 
- `mode:` `ImgMode`
Specifies how an image should fits the space:
  - `fit`: resize the image to fit the box keeping it aspect ratio
  - `cover`: resize the image to cover the box keeping it aspect ratio
  - `stretch`: resize image to take all the space ignoring the aspect ratio
  - `source`: keep image at original size
 
- `modulate:` `Color`
Specifies the color the image should be multiplied

## label

The `<label>` tag is a binable single line of text. It consumes
the children and renders the content of bindable `value` param.


Params:

- `value:` `String`

## progressbar

extends: `<range>`


Params:

from `<range>`
- `minimum:` `f32`
Specifies the minimum value
 
- `maximum:` `f32`
Specifies the maximum value
 
- `value:` `f32`
Specifies absolute value in minimum..maximum range
 
- `relative:` `f32`
Specifies raltive value in 0..1 range
 
- `mode:` `LayoutMode`
Specifies the widget layout arrange.
  
  - `verrtical`: arrange the widget vertically
  - `horizontal`: arrange the widget horisontally

## range

Params:

- `minimum:` `f32`
Specifies the minimum value
 
- `maximum:` `f32`
Specifies the maximum value
 
- `value:` `f32`
Specifies absolute value in minimum..maximum range
 
- `relative:` `f32`
Specifies raltive value in 0..1 range
 
- `mode:` `LayoutMode`
Specifies the widget layout arrange.
  
  - `verrtical`: arrange the widget vertically
  - `horizontal`: arrange the widget horisontally

## slider

extends: `<range>`


Params:

from `<range>`
- `minimum:` `f32`
Specifies the minimum value
 
- `maximum:` `f32`
Specifies the maximum value
 
- `value:` `f32`
Specifies absolute value in minimum..maximum range
 
- `relative:` `f32`
Specifies raltive value in 0..1 range
 
- `mode:` `LayoutMode`
Specifies the widget layout arrange.
  
  - `verrtical`: arrange the widget vertically
  - `horizontal`: arrange the widget horisontally

## span

## strong

The `<strong>` tag highlights an important part of a text. It can be used
for such important contents, as warnings. This can be one sentence that gives
importance to the whole page, or it may be needed if you want to highlight
some words that are of greater importance compared to the rest of the content.


## textinput

Params:

- `value:` `String`

