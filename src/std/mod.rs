mod archive;
mod archive_container;
mod data;
mod index;

pub mod definition;

use alloc::boxed::Box;
use std::io;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use log::debug;

use crate::archive_id::SqPackArchiveId;
use crate::error::Result;
use crate::package::Package;
use crate::reference::SqPackFileReference;

use archive::SqPackArchive;
use archive_container::SqPackArchiveContainer;

pub use index::SqPackIndex;

pub struct SqPackPackage {
    archives: SqPackArchiveContainer,
}

impl SqPackPackage {
    pub fn new(base_dir: &Path) -> io::Result<Self> {
        Ok(Self {
            archives: SqPackArchiveContainer::new(base_dir)?,
        })
    }

    pub async fn archive(&self, archive_id: SqPackArchiveId) -> io::Result<Arc<SqPackArchive>> {
        self.archives.get_archive(archive_id).await
    }
}

#[async_trait]
impl Package for SqPackPackage {
    async fn read_file_by_reference(&self, reference: &SqPackFileReference) -> Result<Vec<u8>> {
        let archive = self.archive(reference.archive_id).await?;

        let result = archive.read_file(reference.hash.folder, reference.hash.file).await;

        #[cfg(debug_assertions)]
        if result.is_err() {
            debug!("No such file {}", reference.path);
        }

        result
    }
}
