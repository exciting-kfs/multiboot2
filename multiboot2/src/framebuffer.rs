use crate::tag_type::Tag;
use crate::Reader;
use core::slice;
use derive_more::Display;

/// The VBE Framebuffer information Tag.
#[derive(Debug, PartialEq, Eq)]
pub struct FramebufferTag<'a> {
    /// Contains framebuffer physical address.
    ///
    /// This field is 64-bit wide but bootloader should set it under 4GiB if
    /// possible for compatibility with payloads which aren’t aware of PAE or
    /// amd64.
    pub address: u64,

    /// Contains the pitch in bytes.
    pub pitch: u32,

    /// Contains framebuffer width in pixels.
    pub width: u32,

    /// Contains framebuffer height in pixels.
    pub height: u32,

    /// Contains number of bits per pixel.
    pub bpp: u8,

    /// The type of framebuffer, one of: `Indexed`, `RGB` or `Text`.
    pub buffer_type: FramebufferType<'a>,
}

/// Helper struct for [`FramebufferType`].
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum FramebufferTypeId {
    Indexed = 0,
    RGB = 1,
    Text = 2,
    // spec says: there may be more variants in the future
}

/// The type of framebuffer.
#[derive(Debug, PartialEq, Eq)]
pub enum FramebufferType<'a> {
    /// Indexed color.
    Indexed {
        #[allow(missing_docs)]
        palette: &'a [FramebufferColor],
    },

    /// Direct RGB color.
    #[allow(missing_docs)]
    #[allow(clippy::upper_case_acronyms)]
    RGB {
        red: FramebufferField,
        green: FramebufferField,
        blue: FramebufferField,
    },

    /// EGA Text.
    ///
    /// In this case the framebuffer width and height are expressed in
    /// characters and not in pixels.
    ///
    /// The bpp is equal 16 (16 bits per character) and pitch is expressed in bytes per text line.
    Text,
}

/// An RGB color type field.
#[derive(Debug, PartialEq, Eq)]
pub struct FramebufferField {
    /// Color field position.
    pub position: u8,

    /// Color mask size.
    pub size: u8,
}

/// A framebuffer color descriptor in the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, packed)] // only repr(C) would add unwanted padding at the end
pub struct FramebufferColor {
    /// The Red component of the color.
    pub red: u8,

    /// The Green component of the color.
    pub green: u8,

    /// The Blue component of the color.
    pub blue: u8,
}

/// Error when an unknown [`FramebufferTypeId`] is found.
#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
#[display(fmt = "Unknown framebuffer type {}", _0)]
pub struct UnknownFramebufferType(u8);

#[cfg(feature = "unstable")]
impl core::error::Error for UnknownFramebufferType {}

/// Transforms a [`Tag`] into a [`FramebufferTag`].
pub fn framebuffer_tag(tag: &Tag) -> Result<FramebufferTag, UnknownFramebufferType> {
    let mut reader = Reader::new(tag as *const Tag);
    reader.skip(8);
    let address = reader.read_u64();
    let pitch = reader.read_u32();
    let width = reader.read_u32();
    let height = reader.read_u32();
    let bpp = reader.read_u8();
    let type_no = reader.read_u8();
    // In the multiboot spec, it has this listed as a u8 _NOT_ a u16.
    // Reading the GRUB2 source code reveals it is in fact a u16.
    reader.skip(2);
    let buffer_type_id = match type_no {
        0 => Ok(FramebufferTypeId::Indexed),
        1 => Ok(FramebufferTypeId::RGB),
        2 => Ok(FramebufferTypeId::Text),
        id => Err(UnknownFramebufferType(id)),
    }?;
    let buffer_type = match buffer_type_id {
        FramebufferTypeId::Indexed => {
            let num_colors = reader.read_u32();
            let palette = unsafe {
                slice::from_raw_parts(
                    reader.current_address() as *const FramebufferColor,
                    num_colors as usize,
                )
            } as &[FramebufferColor];
            FramebufferType::Indexed { palette }
        }
        FramebufferTypeId::RGB => {
            let red_pos = reader.read_u8(); // These refer to the bit positions of the LSB of each field
            let red_mask = reader.read_u8(); // And then the length of the field from LSB to MSB
            let green_pos = reader.read_u8();
            let green_mask = reader.read_u8();
            let blue_pos = reader.read_u8();
            let blue_mask = reader.read_u8();
            FramebufferType::RGB {
                red: FramebufferField {
                    position: red_pos,
                    size: red_mask,
                },
                green: FramebufferField {
                    position: green_pos,
                    size: green_mask,
                },
                blue: FramebufferField {
                    position: blue_pos,
                    size: blue_mask,
                },
            }
        }
        FramebufferTypeId::Text => FramebufferType::Text,
    };

    Ok(FramebufferTag {
        address,
        pitch,
        width,
        height,
        bpp,
        buffer_type,
    })
}
