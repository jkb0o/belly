pub use tagstr_macro::tag;


#[cfg(test)]
mod test {
    use super::*;
    #[test] 
    fn test() {
        assert_eq!(tag!("hello"), tag!("hello"));
        assert_ne!(tag!("hello"), tag!("good bye"));
    }
}