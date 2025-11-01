/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may
 * obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 */

use keramics_core::ErrorTrace;
use keramics_datetime::{DateTime, PosixTime64Ns};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_i32_le, bytes_to_u16_le, bytes_to_u32_le};

use super::attributes_block::ExtAttributesBlock;
use super::constants::*;
use super::inode::ExtInode;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "extra_size", data_type = "u16")),
        member(field(name = "checksum_upper", data_type = "u16", format = "hex")),
        member(field(name = "change_time_extra_precision", data_type = "u32")),
        member(field(name = "modification_time_extra_precision", data_type = "u32")),
        member(field(name = "access_time_extra_precision", data_type = "u32")),
        member(field(name = "creation_time", data_type = "PosixTime32")),
        member(field(name = "creation_time_extra_precision", data_type = "u32")),
        member(field(name = "version_upper", data_type = "u32")),
        member(field(name = "unknown3", data_type = "[u8; 4]")),
    ),
    method(name = "debug_read_data")
)]
/// Extended File System (ext4) inode extension.
pub struct Ext4InodeExtension {}

impl Ext4InodeExtension {
    /// Determines an extra precision timestamp.
    fn get_extra_precision_timestamp(timestamp: i32, mut extra_precision: u32) -> (i64, u32) {
        let mut extra_precision_timestamp: i64 = timestamp as i64;
        if extra_precision > 0 {
            let multiplier: u32 = extra_precision & 0x00000003;

            if multiplier != 0 {
                extra_precision_timestamp += 0x100000000 * (multiplier as i64);
            }
            extra_precision >>= 2;
        }
        (extra_precision_timestamp, extra_precision)
    }

    /// Reads the inode exension from a buffer.
    pub fn read_data(inode: &mut ExtInode, data: &[u8]) -> Result<(), ErrorTrace> {
        let extra_size: u16 = bytes_to_u16_le!(data, 0);

        if extra_size >= 4 {
            let upper_16bit: u16 = bytes_to_u16_le!(data, 2);
            inode.checksum |= (upper_16bit as u32) << 16;
        }
        if inode.flags & 0x00200000 == 0 {
            if extra_size >= 8 {
                let extra_precision: u32 = bytes_to_u32_le!(data, 4);
                let (timestamp, fraction): (i64, u32) =
                    Self::get_extra_precision_timestamp(inode.change_timestamp, extra_precision);

                if timestamp > 0 {
                    inode.change_time =
                        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction));
                }
            }
            if extra_size >= 12 {
                let extra_precision: u32 = bytes_to_u32_le!(data, 8);
                let (timestamp, fraction): (i64, u32) = Self::get_extra_precision_timestamp(
                    inode.modification_timestamp,
                    extra_precision,
                );

                if timestamp > 0 {
                    inode.modification_time =
                        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction));
                }
            }
            if extra_size >= 16 {
                let extra_precision: u32 = bytes_to_u32_le!(data, 12);
                let (timestamp, fraction): (i64, u32) =
                    Self::get_extra_precision_timestamp(inode.access_timestamp, extra_precision);

                if timestamp > 0 {
                    inode.access_time =
                        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction));
                }
            }
        }
        if extra_size >= 24 {
            let timestamp: i32 = bytes_to_i32_le!(data, 16);
            let extra_precision: u32 = bytes_to_u32_le!(data, 20);
            let (extra_precision_timestamp, fraction): (i64, u32) =
                Self::get_extra_precision_timestamp(timestamp, extra_precision);

            if timestamp > 0 {
                inode.creation_time = Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                    extra_precision_timestamp,
                    fraction,
                )));
            }
        }
        let mut data_offset: usize = extra_size as usize;
        let data_end_offset: usize = data_offset + 4;
        let data_size: usize = data.len();

        if data_end_offset < data_size {
            if data[data_offset..data_end_offset] == EXT_ATTRIBUTES_HEADER_SIGNATURE {
                data_offset = data_end_offset;

                let attributes_block: ExtAttributesBlock = ExtAttributesBlock::new();

                match attributes_block.read_entries(
                    data,
                    data_offset,
                    data_size,
                    &mut inode.attributes,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read extended attributes"
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extra_precision_timestamp() -> Result<(), ErrorTrace> {
        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(1733561817, 0);
        assert_eq!(timestamp, 1733561817);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(1733561817, 49382716);
        assert_eq!(timestamp, 1733561817);
        assert_eq!(fraction, 12345679);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-1, 1);
        assert_eq!(timestamp, 4294967295);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-2147483648, 1);
        assert_eq!(timestamp, 2147483648);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(0, 1);
        assert_eq!(timestamp, 4294967296);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(2147483647, 1);
        assert_eq!(timestamp, 6442450943);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-1, 2);
        assert_eq!(timestamp, 8589934591);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-2147483648, 2);
        assert_eq!(timestamp, 6442450944);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(0, 2);
        assert_eq!(timestamp, 8589934592);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(2147483647, 2);
        assert_eq!(timestamp, 10737418239);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-1, 3);
        assert_eq!(timestamp, 12884901887);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(-2147483648, 3);
        assert_eq!(timestamp, 10737418240);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(0, 3);
        assert_eq!(timestamp, 12884901888);
        assert_eq!(fraction, 0);

        let (timestamp, fraction): (i64, u32) =
            Ext4InodeExtension::get_extra_precision_timestamp(2147483647, 3);
        assert_eq!(timestamp, 15032385535);
        assert_eq!(fraction, 0);

        Ok(())
    }

    // TODO: add tests
}
