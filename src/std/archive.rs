use std::io;
use std::path::Path;

use futures::future;
use log::debug;

use super::data::SqPackData;
use super::index::SqPackIndex;
use crate::error::Result;
use crate::raw_file::SqPackRawFile;

pub struct SqPackArchive {
    pub index: SqPackIndex,
    pub data: Vec<SqPackData>,
}

impl SqPackArchive {
    pub async fn new(index_path: &Path) -> io::Result<Self> {
        debug!("Opening {:?}", &index_path);

        let index_path_str = index_path.to_str().unwrap();
        let base_path = index_path_str.trim_end_matches(".index");
        let index = SqPackIndex::new(index_path).await?;

        let futures = (0..index.dat_count()).map(|x| SqPackData::new(base_path, x));
        let data = future::try_join_all(futures).await?;

        Ok(Self { index, data })
    }

    pub fn from_raw(index: Vec<u8>, mut data: Vec<Vec<u8>>) -> Self {
        let index = SqPackIndex::from_raw(index);

        assert!(index.dat_count() as usize == data.len());
        let data = data.drain(..).map(SqPackData::from_raw).collect();

        Self { index, data }
    }

    pub async fn read_raw(&self, folder_hash: u32, file_hash: u32) -> Result<SqPackRawFile> {
        let file_offset = self.index.find_offset(folder_hash, file_hash)?;

        let dat_index = (file_offset & 0x0f) >> 1;
        let offset = ((file_offset as u64) & 0xffff_fff0) << 3;

        Ok(self.data[dat_index as usize].read(offset).await?)
    }

    pub async fn read_file(&self, folder_hash: u32, file_hash: u32) -> Result<Vec<u8>> {
        Ok(self.read_raw(folder_hash, file_hash).await?.into_decoded())
    }

    pub fn folders(&self) -> impl Iterator<Item = u32> + '_ {
        self.index.folders()
    }

    pub fn files(&self, folder_hash: u32) -> Result<impl Iterator<Item = u32> + '_> {
        self.index.files(folder_hash)
    }
}
