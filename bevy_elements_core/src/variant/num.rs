use std::any::TypeId;

use bevy::prelude::warn;

pub fn is_int<T: 'static>() -> bool {
    TypeId::of::<i8>() == TypeId::of::<T>()
        || TypeId::of::<i16>() == TypeId::of::<T>()
        || TypeId::of::<i32>() == TypeId::of::<T>()
        || TypeId::of::<i64>() == TypeId::of::<T>()
        || TypeId::of::<i128>() == TypeId::of::<T>()
        || TypeId::of::<isize>() == TypeId::of::<T>()
        || TypeId::of::<u8>() == TypeId::of::<T>()
        || TypeId::of::<u16>() == TypeId::of::<T>()
        || TypeId::of::<u32>() == TypeId::of::<T>()
        || TypeId::of::<u64>() == TypeId::of::<T>()
        || TypeId::of::<u128>() == TypeId::of::<T>()
        || TypeId::of::<usize>() == TypeId::of::<T>()
}

pub fn is_real<T: 'static>() -> bool {
    TypeId::of::<f32>() == TypeId::of::<T>() || TypeId::of::<f64>() == TypeId::of::<T>()
}

pub fn get_int_ref<T: 'static>(value: &isize) -> Option<&T> {
    if TypeId::of::<i8>() == TypeId::of::<T>() {
        if *value > i8::MAX as isize || *value < i8::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i8", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<i16>() == TypeId::of::<T>() {
        if *value > i16::MAX as isize || *value < i16::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i16", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<i32>() == TypeId::of::<T>() {
        if *value > i32::MAX as isize || *value < i32::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i32", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<i64>() == TypeId::of::<T>() {
        if *value > i64::MAX as isize || *value < i64::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i64", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<i128>() == TypeId::of::<T>() {
        if *value > i128::MAX as isize || *value < i128::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i128", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<isize>() == TypeId::of::<T>() {
        let v = value as *const isize;
        unsafe { v.cast::<T>().as_ref() }
    } else if TypeId::of::<u8>() == TypeId::of::<T>() {
        if *value > u8::MAX as isize || *value < u8::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u8", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<u16>() == TypeId::of::<T>() {
        if *value > u16::MAX as isize || *value < u16::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u16", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<u32>() == TypeId::of::<T>() {
        if *value > u32::MAX as isize || *value < u32::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u32", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<u64>() == TypeId::of::<T>() {
        if *value > u64::MAX as isize || *value < u64::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u64", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<u128>() == TypeId::of::<T>() {
        if *value < 0 {
            warn!("Attempt to cast invalid value ({}) to u128", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else if TypeId::of::<usize>() == TypeId::of::<T>() {
        if *value < 0 {
            warn!("Attempt to cast invalid value ({}) to u64", value);
            None
        } else {
            let v = value as *const isize;
            unsafe { v.cast::<T>().as_ref() }
        }
    } else {
        None
    }
}

pub fn get_real_ref<T: 'static>(value: &f64) -> Option<&T> {
    if TypeId::of::<f32>() == TypeId::of::<T>() {
        let v = value as *const f64;
        unsafe { v.cast::<T>().as_ref() }
    } else if TypeId::of::<f64>() == TypeId::of::<T>() {
        let v = value as *const f64;
        unsafe { v.cast::<T>().as_ref() }
    } else {
        None
    }
}

pub fn get_int_mut<T: 'static>(value: &mut isize) -> Option<&mut T> {
    if TypeId::of::<i8>() == TypeId::of::<T>() {
        if *value > i8::MAX as isize || *value < i8::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i8", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<i16>() == TypeId::of::<T>() {
        if *value > i16::MAX as isize || *value < i16::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i16", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<i32>() == TypeId::of::<T>() {
        if *value > i32::MAX as isize || *value < i32::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i32", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<i64>() == TypeId::of::<T>() {
        if *value > i64::MAX as isize || *value < i64::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i64", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<i128>() == TypeId::of::<T>() {
        if *value > i128::MAX as isize || *value < i128::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to i128", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<isize>() == TypeId::of::<T>() {
        let v = value as *mut isize;
        unsafe { v.cast::<T>().as_mut() }
    } else if TypeId::of::<u8>() == TypeId::of::<T>() {
        if *value > u8::MAX as isize || *value < u8::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u8", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<u16>() == TypeId::of::<T>() {
        if *value > u16::MAX as isize || *value < u16::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u16", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<u32>() == TypeId::of::<T>() {
        if *value > u32::MAX as isize || *value < u32::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u32", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<u64>() == TypeId::of::<T>() {
        if *value > u64::MAX as isize || *value < u64::MIN as isize {
            warn!("Attempt to cast invalid value ({}) to u64", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<u128>() == TypeId::of::<T>() {
        if *value < 0 {
            warn!("Attempt to cast invalid value ({}) to u128", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else if TypeId::of::<usize>() == TypeId::of::<T>() {
        if *value < 0 {
            warn!("Attempt to cast invalid value ({}) to u64", value);
            None
        } else {
            let v = value as *mut isize;
            unsafe { v.cast::<T>().as_mut() }
        }
    } else {
        None
    }
}

pub fn get_real_mut<T: 'static>(value: &mut f64) -> Option<&mut T> {
    if TypeId::of::<f32>() == TypeId::of::<T>() {
        let v = value as *mut f64;
        unsafe { v.cast::<T>().as_mut() }
    } else if TypeId::of::<f64>() == TypeId::of::<T>() {
        let v = value as *mut f64;
        unsafe { v.cast::<T>().as_mut() }
    } else {
        None
    }
}

pub fn get_int<T: 'static>(mut value: isize) -> Option<T> {
    get_int_mut::<T>(&mut value).map(|v| {
        let v = v as *mut T;
        unsafe { *Box::from_raw(v) }
    })
}

pub fn get_real<T: 'static>(mut value: f64) -> Option<T> {
    get_real_mut::<T>(&mut value).map(|v| {
        let v = v as *mut T;
        unsafe { *Box::from_raw(v) }
    })
}
