use bsx::*;

#[test]
fn test_valid_syntax() {
    bsx! { 
        <box>
            <h1>"Hello!"</h1>
        </box> 
    };
}