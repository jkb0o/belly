use std::{marker::PhantomData, str::FromStr};

use crate::TransformationError;

// See explanation here:
// https://github.com/dtolnay/case-studies/tree/master/autoref-specialization#realistic-application

pub struct ConvertFromStr;
pub trait TryConvertFromStr {
    #[inline]
    fn convert_anyhow(&self) -> ConvertFromStr {
        ConvertFromStr
    }
}
impl TryConvertFromStr for &str {}
impl TryConvertFromStr for String {}
impl ConvertFromStr {
    pub fn convert<S: AsRef<str>, T: FromStr<Err = E>, E: Into<TransformationError>>(
        self,
        source: S,
    ) -> Result<T, TransformationError> {
        T::from_str(source.as_ref()).map_err(|e| e.into())
    }
}

pub struct ConvertTryFrom<S>(PhantomData<S>);
pub trait TryConvertTryFrom<T> {
    #[inline]
    fn convert_anyhow(&self) -> ConvertTryFrom<T> {
        ConvertTryFrom(PhantomData)
    }
}
impl<S, T: From<S>> TryConvertTryFrom<S> for &T {}
impl<S> ConvertTryFrom<S> {
    pub fn convert<T: TryFrom<S, Error = E>, E: Into<TransformationError>>(
        self,
        source: S,
    ) -> Result<T, TransformationError> {
        T::try_from(source).map_err(|e| e.into())
    }
}

#[macro_export]
macro_rules! convert {
    ($val:expr) => {{
        #[allow(unused_imports)]
        use $crate::relations::convert::{
            ConvertFromStr, ConvertTryFrom, TryConvertFromStr, TryConvertTryFrom,
        };
        match $val {
            val => (&val).convert_anyhow().convert(val),
        }
    }};
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    #[derive(Default)]
    struct LikeF32(f32);
    impl TryFrom<LikeF32> for f32 {
        type Error = String;
        fn try_from(value: LikeF32) -> Result<Self, Self::Error> {
            Ok(value.0)
        }
    }
    impl FromStr for LikeF32 {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(LikeF32(
                s.parse().map_err(|e| format!("Can't parse {e} as f32"))?,
            ))
        }
    }

    #[derive(Default)]
    struct Range {
        value: f32,
        num_value: LikeF32,
    }

    #[test]
    fn test_convertion() {
        let mut range = Range::default();
        let s0 = "0.";
        let s1 = 1.;
        let s2 = "2.".to_string();
        let s3 = LikeF32(3.);
        let s4 = "4.";
        if let Ok(v) = convert!(s0) {
            range.value = v;
        }
        assert_eq!(range.value, 0.);
        if let Ok(v) = convert!(s1) {
            range.value = v;
        }
        assert_eq!(range.value, 1.);
        if let Ok(v) = convert!(s2) {
            range.value = v;
        }
        assert_eq!(range.value, 2.);
        let mut error = None;
        match convert!(s3) {
            Ok(v) => range.value = v,
            Err(e) => error = Some(format!("Can't convert: {}", e.as_str())),
        }
        assert_eq!(range.value, 3.);
        assert_eq!(error, None);

        if let Ok(v) = convert!(s4) {
            range.num_value = v;
        }
        assert_eq!(range.num_value.0, 4.)
    }
}
