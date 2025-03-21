//! Module for [CommandLineTag].

use crate::{Tag, TagTrait, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::str;

/// This tag contains the command line string.
///
/// The string is a normal C-style UTF-8 zero-terminated string that can be
/// obtained via the `command_line` method.
#[repr(C, packed)] // only repr(C) would add unwanted padding before first_section
#[derive(ptr_meta::Pointee)]
pub struct CommandLineTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    cmdline: [u8],
}

impl CommandLineTag {
    /// Reads the command line of the kernel as Rust string slice without
    /// the null-byte.
    ///
    /// For example, this returns `"console=ttyS0"`.if the GRUB config
    /// contains  `"multiboot2 /mykernel console=ttyS0"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # let boot_info = unsafe { multiboot2::load(0xdeadbeef).unwrap() };
    /// if let Some(tag) = boot_info.command_line_tag() {
    ///     let command_line = tag.command_line();
    ///     assert_eq!(Ok("/bootarg"), command_line);
    /// }
    /// ```
    pub fn command_line(&self) -> Result<&str, str::Utf8Error> {
        Tag::get_dst_str_slice(&self.cmdline)
    }
}

impl Debug for CommandLineTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommandLineTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("cmdline", &self.command_line())
            .finish()
    }
}

impl TagTrait for CommandLineTag {
    fn dst_size(base_tag: &Tag) -> usize {
        // The size of the sized portion of the command line tag.
        let tag_base_size = 8;
        assert!(base_tag.size >= 8);
        base_tag.size as usize - tag_base_size
    }
}

#[cfg(test)]
mod tests {
    use crate::{CommandLineTag, Tag, TagType};

    const MSG: &str = "hello";

    /// Returns the tag structure in bytes in native endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        // size is: 4 bytes for tag + 4 bytes for size + length of null-terminated string
        let size = (4 + 4 + MSG.as_bytes().len() + 1) as u32;
        [
            &((TagType::Cmdline.val()).to_ne_bytes()),
            &size.to_ne_bytes(),
            MSG.as_bytes(),
            // Null Byte
            &[0],
        ]
        .iter()
        .flat_map(|bytes| bytes.iter())
        .copied()
        .collect()
    }

    /// Tests to parse a string with a terminating null byte from the tag (as the spec defines).
    #[test]
    fn test_parse_str() {
        let tag = get_bytes();
        let tag = unsafe { &*tag.as_ptr().cast::<Tag>() };
        let tag = tag.cast_tag::<CommandLineTag>();
        assert_eq!({ tag.typ }, TagType::Cmdline);
        assert_eq!(tag.command_line().expect("must be valid UTF-8"), MSG);
    }
}
