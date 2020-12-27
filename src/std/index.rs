use core::mem::size_of;
use std::io;
use std::path::Path;

use async_std::fs;

use super::definition::{FileSegment, FolderSegment, SqPackHeader, SqPackIndexHeader};
use crate::error::{Result, SqPackReaderError};
use crate::util::{cast, cast_array, cast_mut};

pub struct SqPackIndex {
    data: Vec<u8>,
}

impl SqPackIndex {
    pub async fn new(path: &Path) -> io::Result<Self> {
        let data = fs::read(path).await?;

        Ok(Self::from_raw(data))
    }

    pub fn from_raw(raw: Vec<u8>) -> Self {
        Self { data: raw }
    }

    pub fn dat_count(&self) -> u32 {
        let sqpack_header = cast::<SqPackHeader>(&self.data);
        let index_header = cast::<SqPackIndexHeader>(&self.data[sqpack_header.header_length as usize..]);

        index_header.dat_count
    }

    pub fn find_offset(&self, folder_hash: u32, file_hash: u32) -> Result<u32> {
        Ok(self.find_file_segment(folder_hash, file_hash)?.data_offset)
    }

    pub fn folders(&self) -> impl Iterator<Item = u32> + '_ {
        let folder_segments = self.get_folder_segments();

        folder_segments.iter().map(|x| x.folder_hash)
    }

    pub fn files(&self, folder_hash: u32) -> Result<impl Iterator<Item = u32> + '_> {
        let folder_segments = self.get_folder_segments();

        let folder_index = folder_segments
            .binary_search_by_key(&folder_hash, |x| x.folder_hash)
            .map_err(|_| SqPackReaderError::NoSuchFolder)?;
        let folder = &folder_segments[folder_index];

        let file_segments = self.get_file_segments(folder);

        Ok(file_segments.iter().map(|x| x.file_hash))
    }

    pub fn write_offset(&mut self, folder_hash: u32, file_hash: u32, new_offset: u32) -> Result<()> {
        let segment = self.find_file_segment(folder_hash, file_hash)?;

        // XXX
        #[allow(mutable_transmutes)]
        #[allow(clippy::transmute_ptr_to_ptr)]
        let mut segment: &mut FileSegment = unsafe { core::mem::transmute(segment) };
        segment.data_offset = new_offset;

        Ok(())
    }

    pub fn write_dat_count(&mut self, new_dat_count: u32) {
        let header_length = cast::<SqPackHeader>(&self.data).header_length;
        let index_header = cast_mut::<SqPackIndexHeader>(&mut self.data[header_length as usize..]);

        index_header.dat_count = new_dat_count
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn get_folder_segments(&self) -> &[FolderSegment] {
        let sqpack_header = cast::<SqPackHeader>(&self.data);
        let index_header = cast::<SqPackIndexHeader>(&self.data[sqpack_header.header_length as usize..]);

        let segment_count = index_header.folder_segment.size as usize / size_of::<FolderSegment>();

        &cast_array::<FolderSegment>(&self.data[index_header.folder_segment.offset as usize..])[..segment_count]
    }

    fn get_file_segments(&self, folder: &FolderSegment) -> &[FileSegment] {
        let sqpack_header = cast::<SqPackHeader>(&self.data);
        let index_header = cast::<SqPackIndexHeader>(&self.data[sqpack_header.header_length as usize..]);

        let file_begin = (folder.file_list_offset - index_header.file_segment.offset) as usize / size_of::<FileSegment>();
        let file_end = file_begin + folder.file_list_size as usize / size_of::<FileSegment>();

        &cast_array::<FileSegment>(&self.data[index_header.file_segment.offset as usize..])[file_begin..file_end]
    }

    fn find_file_segment(&self, folder_hash: u32, file_hash: u32) -> Result<&FileSegment> {
        let folder_segments = self.get_folder_segments();
        let folder_index = folder_segments
            .binary_search_by_key(&folder_hash, |x| x.folder_hash)
            .map_err(|_| SqPackReaderError::NoSuchFolder)?;
        let folder = &folder_segments[folder_index];

        let file_segments = self.get_file_segments(folder);
        let file_index = file_segments
            .binary_search_by_key(&file_hash, |x| x.file_hash)
            .map_err(|_| SqPackReaderError::NoSuchFile)?;

        Ok(&file_segments[file_index])
    }
}
