use bevy_elements_macro::*;

#[test]
fn test_valid_syntax() {
    eml! {
        <box>
            <h1>"Hello!"</h1>
        </box>
    };
}
