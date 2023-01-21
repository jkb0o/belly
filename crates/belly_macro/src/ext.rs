pub trait CapitalizeExt {
    fn capitalize(&self) -> String;
    fn capitalized(&self) -> bool;
}

impl<T: AsRef<str>> CapitalizeExt for T {
    fn capitalize(&self) -> String {
        let mut c = self.as_ref().chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
    fn capitalized(&self) -> bool {
        if self.as_ref().is_empty() {
            return false;
        }
        self.as_ref().chars().next().unwrap().is_uppercase()
    }
}

pub trait ToCamelCaseExt {
    fn to_camel_case(&self) -> String;
}

impl<T: AsRef<str>> ToCamelCaseExt for T {
    fn to_camel_case(&self) -> String {
        self.as_ref()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|c| !c.is_empty())
            .map(|c| c.to_lowercase().capitalize())
            .collect()
    }
}
