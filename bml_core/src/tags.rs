pub use tagstr::*;

pub mod tags {
    use super::*;
    pub fn params() -> Tag {
        tag!("params")
    }
    pub fn class() -> Tag {
        tag!("class")
    }

    pub fn styles() -> Tag {
        tag!("styles")
    }

    pub fn id() -> Tag {
        tag!("id")
    }

    pub fn with() -> Tag {
        tag!("with")
    }
}