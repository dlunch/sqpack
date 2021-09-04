use alloc::vec::Vec;

use bytes::Bytes;
use miniz_oxide::inflate::decompress_to_vec;

use crate::util::cast;

#[repr(C)]
pub struct BlockHeader {
    pub header_size: u32,
    _unk: u32,
    pub compressed_length: u32, // 32000 if not compressed
    pub uncompressed_length: u32,
}

pub struct SqPackRawFile {
    pub uncompressed_size: u32,
    pub header: Bytes,
    pub blocks: Vec<Bytes>,
}

impl SqPackRawFile {
    pub fn from_blocks(uncompressed_size: u32, header: Bytes, blocks: Vec<Bytes>) -> Self {
        Self {
            uncompressed_size,
            header,
            blocks,
        }
    }

    pub fn into_decoded(self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.uncompressed_size as usize + self.header.len());
        result.extend(self.header);
        if result.len() == 4 {
            result.resize(result.len() + 0x40, 0); // mdl files has 0x44 bytes of header
        }

        for block in self.blocks {
            Self::decode_block_into(&block, &mut result);
        }

        result
    }

    pub fn get_block_header(block: &[u8]) -> &BlockHeader {
        cast::<BlockHeader>(block)
    }

    fn decode_block_into(block: &[u8], result: &mut Vec<u8>) {
        let header = Self::get_block_header(block);

        if header.compressed_length >= 32000 {
            result.extend(&block[header.header_size as usize..header.header_size as usize + header.uncompressed_length as usize]);
        } else {
            let data = &block[header.header_size as usize..header.header_size as usize + header.compressed_length as usize];

            result.extend(decompress_to_vec(data).unwrap());
        }
    }
}
