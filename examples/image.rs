use bevy::prelude::*;
use bevy_elements::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let img = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:justify-content="center" s:padding="20px">
            <img {img} src="icon.png" mode="fit" s:width="50%" s:height="70%" s:background-color="grey" s:margin-bottom="20px"/>
            <br/>
            // <buttongroup bind:value=to!(img, Img.mode) 
            <div s:max-width="600px" s:align-content="center" s:align-items="center">
                <span s:min-width="80px">"Mode:"</span>
                <button mode="group(mode)" s:flex-grow="1.0" on:press=connect!(img, |i:Img| i.mode = ImgMode::Fit)>"fit"</button>
                <button mode="group(mode)" s:flex-grow="1.0" on:press=connect!(img, |i:Img| i.mode = ImgMode::Cover)>"cover"</button>
                <button mode="group(mode)" s:flex-grow="1.0" on:press=connect!(img, |i:Img| i.mode = ImgMode::Stretch)>"stretch"</button>
                <button mode="group(mode)" s:flex-grow="1.0" on:press=connect!(img, |i:Img| i.mode = ImgMode::Source)>"source"</button>
            </div>
            <br/>
            <buttongroup value=bind!(=> img, Img.src) s:width="600px" s:align-content="center" s:align-items="center">
                <span s:min-width="80px">"Source:"</span>
                <button s:flex-grow="1.0" value="icon.png">"icon.png"</button>
                <button s:flex-grow="1.0" value="bevy_logo_light.png">"bevy_logo_light.png"</button>
                <button pressed s:flex-grow="1.0" value="bevy_logo_dark.png">"bevy_logo_dark.png"</button>
            </buttongroup>
        </body>
    });
}
