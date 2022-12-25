*There is no way to feel the happiness without understanding the pain.*

About
-----

`Elements` is a plugin for `bevy` game engine that helps to declaratively define a user interface with `eml` markup (macros & asset), a very-css-like `ess` syntax, and specifying relationships between data using `connect!` & `bind!`.

The main tasks the plugin is about to solve are:
- fill the spaces in the `bevy` UI system (inputs, scrolls, text layout, etc.)
- reduce the [boilerplate](https://bevyengine.org/examples/ui/ui/) defining the UI
- allow to effectively separate the layout, styling, data & logic from each other
- build a base to provide various tools for *`game`* developers & designers

During plugin development I am guided by the following values:
- type safity: *about to explain*
- namespace safity: *about to explain*
- no compromises: *about to explain*
- on the top of `bevy`: *about to explain*
- performance matters: *about to explain*
- beauty matters as well: *about to explain*
- tooling rocks: *about to explain*

Prerequisites & Setup
---------------------

If you land here, it means you have some experience using `rust` & `cargo` and you already have a `bevy`-based project. The only step you need is:
```
cargo add belly
```

If you want just to play around it is better to get the sources & run the [examples](TODO):
```
git clone https://github.com/jkb0o/belly.git
cd belly
cargo run --example hello_world
```

Basics
------

Not another word! Let's write code:

```rust
use bevy::prelude::*;
use belly::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(eml! {
        <body s:padding="50px">
            "Hello, "<strong>"world"</strong>"!"
        </body>
    });
}
```
![Hello world example](docs/hello_world.png "Hello world example screenshot")




Styling
-------
*I've paid my dues*

*Time after time*


Binds
-----
*I've done my sentence*

*But committed no crime*

Events
------
*And bad mistakes*

*I've made a few*


Widgets
-------
*I've had my share of sand*

*Kicked in my face*


Style properties
----------------

*But I've come through*


Examples
--------

*And we mean to go on and on and on and on*

Roadmap
-------

*We are the champions, my friends...*


*And we'll keep on fighting till the end!!!*









