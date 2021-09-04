use std::io;
use std::path::PathBuf;

use async_std::task;
use async_trait::async_trait;
use cfg_if::cfg_if;

#[async_trait]
pub trait File: Sync + Send {
    async fn read_at(&self, offset: u64, length: usize) -> io::Result<Vec<u8>>;
}

pub struct FileImpl {
    file: std::fs::File,
}

impl FileImpl {
    pub async fn open(path: PathBuf) -> io::Result<Self> {
        let file = task::spawn_blocking(move || std::fs::File::open(path)).await?;

        Ok(Self { file })
    }
}

#[async_trait]
impl File for FileImpl {
    async fn read_at(&self, offset: u64, length: usize) -> io::Result<Vec<u8>> {
        // XXX
        let file: &std::fs::File = unsafe { core::mem::transmute(&self.file) };

        task::spawn_blocking(move || {
            let mut buf = vec![0; length];

            cfg_if! {
                if #[cfg(unix)] {
                    use std::os::unix::fs::FileExt;

                    file.read_exact_at(&mut buf, offset)?
                }
                else if #[cfg(windows)] {
                    use std::os::windows::fs::FileExt;

                    file.seek_read(&mut buf, offset)?
                }
            }

            Ok(buf)
        })
        .await
    }
}
