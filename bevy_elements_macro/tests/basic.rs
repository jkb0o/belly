use bevy_elements_macro::*;

#[test]
fn test_valid_syntax() {
    bsx! { 
        <box>
            <h1>"Hello!"</h1>
        </box> 
    };
}