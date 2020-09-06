#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

mod archive_id;
mod definition;
mod error;
mod package;
mod raw_file;
mod reference;
mod util;

pub use archive_id::SqPackArchiveId;
pub use error::{Result, SqPackReaderError};
pub use package::Package;
pub use reference::{SqPackFileHash, SqPackFileReference};

pub mod internal {
    mod definition {
        pub use crate::definition::*;
    }
    pub use crate::raw_file::SqPackRawFile;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        mod sqpack_reader;
        pub use sqpack_reader::SqPackReader;
    }
}
