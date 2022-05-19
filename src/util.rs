cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub fn cast_mut<T>(data: &mut [u8]) -> &mut T {
            unsafe { &mut *(data.as_mut_ptr() as *mut T) }
        }

        pub fn cast_array<T>(data: &[u8]) -> &[T] {
            unsafe { &*(data as *const [u8] as *const [T]) }
        }
    }
}

pub fn cast<T>(data: &[u8]) -> &T {
    unsafe { &*(data.as_ptr() as *const T) }
}
