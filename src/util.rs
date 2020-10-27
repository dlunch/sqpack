cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        use std::io;
        use std::io::SeekFrom;

        use async_std::fs::File;
        use async_std::io::prelude::ReadExt as async_std_read_ext;
        use async_std::io::prelude::SeekExt;
        use async_trait::async_trait;

        #[async_trait]
        pub trait ReadExt {
            async fn read_bytes(&mut self, offset: u64, size: usize) -> io::Result<Vec<u8>>;
        }

        #[async_trait]
        impl ReadExt for File {
            async fn read_bytes(&mut self, offset: u64, size: usize) -> io::Result<Vec<u8>> {
                let mut data = vec![0; size];
                self.seek(SeekFrom::Start(offset)).await?;
                self.read_exact(&mut data).await?;

                Ok(data)
            }
        }

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
