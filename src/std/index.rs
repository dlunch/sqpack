use core::mem::size_of;
use std::io;
use std::path::Path;

use async_std::fs;

use crate::definition::{FileSegment, FolderSegment, SqPackHeader, SqPackIndexHeader};
use crate::error::{Result, SqPackReaderError};
use crate::util::{cast, cast_array};

pub struct SqPackIndex {
    data: Vec<u8>,
}

impl SqPackIndex {
    pub async fn new(path: &Path) -> io::Result<Self> {
        let data = fs::read(path).await?;

        Ok(Self { data })
    }

    pub fn dat_count(&self) -> u32 {
        let sqpack_header = cast::<SqPackHeader>(&self.data);
        let index_header = cast::<SqPackIndexHeader>(&self.data[sqpack_header.header_length as usize..]);

        index_header.dat_count
    }

    pub fn find_offset(&self, folder_hash: u32, file_hash: u32) -> Result<u32> {
        let folder_segments = self.get_folder_segments();
        let folder_index = folder_segments
            .binary_search_by_key(&folder_hash, |x| x.folder_hash)
            .map_err(|_| SqPackReaderError::NoSuchFolder)?;
        let folder = &folder_segments[folder_index];

        let file_segments = self.get_file_segments(folder);
        let file_index = file_segments
            .binary_search_by_key(&file_hash, |x| x.file_hash)
            .map_err(|_| SqPackReaderError::NoSuchFile)?;
        let file = &file_segments[file_index];

        Ok(file.data_offset)
    }

    pub fn folders<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        let folder_segments = self.get_folder_segments();

        folder_segments.iter().map(|x| x.folder_hash)
    }

    pub fn files<'a>(&'a self, folder_hash: u32) -> Result<impl Iterator<Item = u32> + 'a> {
        let folder_segments = self.get_folder_segments();

        let folder_index = folder_segments
            .binary_search_by_key(&folder_hash, |x| x.folder_hash)
            .map_err(|_| SqPackReaderError::NoSuchFolder)?;
        let folder = &folder_segments[folder_index];

        let file_segments = self.get_file_segments(folder);

        Ok(file_segments.iter().map(|x| x.file_hash))
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
}
