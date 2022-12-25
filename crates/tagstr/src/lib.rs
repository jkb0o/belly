pub use tagstr_core::{tag, AsTag, Tag};
// pub use tagstr_macro::tag;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_macro() {
        assert_eq!(tag!("hello"), tag!("hello"));
        assert_ne!(tag!("hello"), tag!("good bye"));
    }

    fn test_tag() -> Tag {
        tag!("test")
    }
    #[test]
    fn test_mixed_equals() {
        assert_eq!("test".as_tag(), test_tag());
    }
}
