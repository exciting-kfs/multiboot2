use crate::{Tag, TagTrait, TagTypeId};

use core::fmt::Debug;

const METADATA_SIZE: usize = core::mem::size_of::<TagTypeId>()
    + core::mem::size_of::<u32>()
    + core::mem::size_of::<u8>() * 8;

/// This tag contains a copy of SMBIOS tables as well as their version.
#[derive(ptr_meta::Pointee)]
#[repr(C, packed)]
pub struct SmbiosTag {
    typ: TagTypeId,
    size: u32,
    pub major: u8,
    pub minor: u8,
    _reserved: [u8; 6],
    pub tables: [u8],
}

impl TagTrait for SmbiosTag {
    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

impl Debug for SmbiosTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("major", &{ self.major })
            .field("minor", &{ self.minor })
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{SmbiosTag, Tag, TagType};

    /// Returns the tag structure in bytes in native endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        let tables = [0xabu8; 24];
        // size is: 4 bytes for tag + 4 bytes for size + 1 byte for major and minor
        // + 6 bytes reserved + the actual tables
        let size = (4 + 4 + 1 + 1 + 6 + tables.len()) as u32;
        let typ: u32 = TagType::Smbios.into();
        let mut bytes = [typ.to_ne_bytes(), size.to_ne_bytes()].concat();
        bytes.push(3);
        bytes.push(0);
        bytes.extend([0; 6]);
        bytes.extend(tables);
        bytes
    }

    /// Test to parse a given tag.
    #[test]
    fn test_parse() {
        let tag = get_bytes();
        let tag = unsafe { &*tag.as_ptr().cast::<Tag>() };
        let tag = tag.cast_tag::<SmbiosTag>();
        assert_eq!({ tag.typ }, TagType::Smbios);
        assert_eq!(tag.major, 3);
        assert_eq!(tag.minor, 0);
        assert_eq!(tag.tables, [0xabu8; 24]);
    }
}
