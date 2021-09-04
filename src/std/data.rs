use core::convert::TryInto;
use core::mem::size_of;
use std::io;
use std::path::PathBuf;

use bytes::Bytes;
use log::{debug, trace};

use super::definition::{DefaultFrameInfo, FileHeader, FileType, ImageFrameInfo, ModelFrameInfo};
use super::file::{File, FileImpl};
use crate::raw_file::SqPackRawFile;
use crate::util::cast;

pub struct SqPackData {
    file: Box<dyn File>,
}

impl SqPackData {
    pub async fn new(base_path: &str, index: u32) -> io::Result<Self> {
        let file_path = PathBuf::from(format!("{}.dat{}", base_path, index));

        debug!("Opening {:?}", &file_path);

        let file = Box::new(FileImpl::open(file_path).await?);

        Ok(Self { file })
    }

    pub async fn read(&self, offset: u64) -> io::Result<SqPackRawFile> {
        trace!("Read from offset {}", offset);

        let file_header_data = self.file.read_at(offset, size_of::<FileHeader>()).await?;
        let file_header = cast::<FileHeader>(&file_header_data);
        match FileType::from_raw(file_header.file_type) {
            FileType::Default => Ok(self.read_default(offset, file_header).await?),
            FileType::Model => Ok(self.read_model(offset, file_header).await?),
            FileType::Image => Ok(self.read_image(offset, file_header).await?),
        }
    }

    async fn read_default(&self, base_offset: u64, file_header: &FileHeader) -> io::Result<SqPackRawFile> {
        trace!("Read default from offset {}", base_offset);

        let frame_infos_data = self
            .file
            .read_at(
                base_offset + size_of::<FileHeader>() as u64,
                size_of::<DefaultFrameInfo>() * file_header.frame_count as usize,
            )
            .await?;
        let frame_infos = (0..file_header.frame_count as usize)
            .map(|x| cast::<DefaultFrameInfo>(&frame_infos_data[x * size_of::<DefaultFrameInfo>()..]))
            .collect::<Vec<_>>();

        let mut blocks = Vec::with_capacity(frame_infos.len());
        for frame_info in frame_infos {
            let offset = base_offset + file_header.header_length as u64 + frame_info.block_offset as u64;
            let block = self.file.read_at(offset, frame_info.block_size as usize).await?;

            blocks.push(block.into());
        }

        Ok(SqPackRawFile::from_blocks(file_header.uncompressed_size, Bytes::new(), blocks))
    }

    async fn read_block_sizes(&self, offset: u64, count: usize) -> io::Result<Vec<u16>> {
        let item_size = size_of::<u16>();
        let block_size_data = Bytes::from(self.file.read_at(offset, count * item_size).await?);

        Ok((0..count * item_size)
            .step_by(item_size)
            .map(|x| u16::from_le_bytes(block_size_data[x..x + size_of::<u16>()].try_into().unwrap()))
            .collect::<Vec<_>>())
    }

    async fn read_contiguous_blocks(&self, base_offset: u64, block_sizes: &[u16]) -> io::Result<Bytes> {
        let total_size = block_sizes.iter().map(|&x| x as usize).sum();

        Ok(self.file.read_at(base_offset, total_size).await?.into())
    }

    async fn read_model(&self, base_offset: u64, file_header: &FileHeader) -> io::Result<SqPackRawFile> {
        trace!("Read model from offset {}", base_offset);

        let frame_info_data = self
            .file
            .read_at(base_offset + size_of::<FileHeader>() as u64, size_of::<ModelFrameInfo>())
            .await?;
        let frame_info = cast::<ModelFrameInfo>(&frame_info_data);

        let mut header = Vec::with_capacity(size_of::<u16>() * 2);
        header.extend(frame_info.number_of_meshes.to_le_bytes().iter());
        header.extend(frame_info.number_of_materials.to_le_bytes().iter());

        let total_block_count = frame_info.block_counts.iter().sum::<u16>() as usize;
        let sizes_offset = base_offset + size_of::<FileHeader>() as u64 + size_of::<ModelFrameInfo>() as u64;
        let block_sizes = self.read_block_sizes(sizes_offset, total_block_count).await?;

        let block_base_offset = base_offset + file_header.header_length as u64 + frame_info.offsets[0] as u64;
        let block_data = self.read_contiguous_blocks(block_base_offset, &block_sizes).await?;

        let blocks = Self::iterate_blocks(block_data, block_sizes).collect::<Vec<_>>();

        Ok(SqPackRawFile::from_blocks(file_header.uncompressed_size, header.into(), blocks))
    }

    async fn read_image(&self, base_offset: u64, file_header: &FileHeader) -> io::Result<SqPackRawFile> {
        trace!("Read image from offset {}", base_offset);

        let frame_infos_data = self
            .file
            .read_at(
                base_offset + size_of::<FileHeader>() as u64,
                size_of::<ImageFrameInfo>() * file_header.frame_count as usize,
            )
            .await?;
        let frame_infos = (0..file_header.frame_count as usize)
            .map(|x| cast::<ImageFrameInfo>(&frame_infos_data[x * size_of::<ImageFrameInfo>()..]))
            .collect::<Vec<_>>();

        let sizes_table_base = base_offset + size_of::<FileHeader>() as u64 + file_header.frame_count as u64 * size_of::<ImageFrameInfo>() as u64;

        let header = self
            .file
            .read_at(base_offset + file_header.header_length as u64, frame_infos[0].block_offset as usize)
            .await?;

        let mut contiguous_blocks = Vec::with_capacity(frame_infos.len());
        for frame_info in frame_infos {
            let block_sizes = self
                .read_block_sizes(
                    sizes_table_base + frame_info.sizes_table_offset as u64 * size_of::<u16>() as u64,
                    frame_info.block_count as usize,
                )
                .await?;

            let block_base_offset = base_offset + file_header.header_length as u64 + frame_info.block_offset as u64;
            let block_data = self.read_contiguous_blocks(block_base_offset, &block_sizes).await?;

            contiguous_blocks.push((block_data, block_sizes));
        }

        let blocks = contiguous_blocks
            .into_iter()
            .flat_map(|(block_data, block_sizes)| Self::iterate_blocks(block_data, block_sizes))
            .collect::<Vec<_>>();

        Ok(SqPackRawFile::from_blocks(file_header.uncompressed_size, header.into(), blocks))
    }

    fn iterate_blocks(block_data: Bytes, block_sizes: Vec<u16>) -> impl Iterator<Item = Bytes> {
        block_sizes.into_iter().scan(0usize, move |offset, block_size| {
            let result = block_data.slice(*offset..);
            *offset += block_size as usize;

            Some(result)
        })
    }
}
