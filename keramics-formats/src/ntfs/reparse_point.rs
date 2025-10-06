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
use keramics_core::mediator::Mediator;

use super::constants::*;
use super::mft_attribute::NtfsMftAttribute;
use super::reparse_point_header::NtfsReparsePointHeader;
use super::symbolic_link_reparse_data::NtfsSymoblicLinkReparseData;
use super::wof_reparse_data::NtfsWofReparseData;

/// New Technologies File System (NTFS) reparse point.
pub enum NtfsReparsePoint {
    SymbolicLink {
        reparse_data: NtfsSymoblicLinkReparseData,
    },
    Undefined {
        tag: u32,
    },
    WindowsOverlayFilter {
        reparse_data: NtfsWofReparseData,
    },
}

impl NtfsReparsePoint {
    /// Creates a new reparse point.
    pub(super) fn new(tag: u32) -> Self {
        match tag {
            0x80000017 => NtfsReparsePoint::WindowsOverlayFilter {
                reparse_data: NtfsWofReparseData::new(),
            },
            0xa000000c => NtfsReparsePoint::SymbolicLink {
                reparse_data: NtfsSymoblicLinkReparseData::new(),
            },
            _ => NtfsReparsePoint::Undefined { tag: tag },
        }
    }

    /// Retrieves the reparse tag.
    pub fn get_reparse_tag(&self) -> u32 {
        match self {
            NtfsReparsePoint::SymbolicLink { .. } => 0xa000000c,
            NtfsReparsePoint::Undefined { tag } => *tag,
            NtfsReparsePoint::WindowsOverlayFilter { .. } => 0x80000017,
        }
    }

    /// Reads the reparse data from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        match self {
            NtfsReparsePoint::SymbolicLink { reparse_data } => reparse_data.read_data(data),
            NtfsReparsePoint::WindowsOverlayFilter { reparse_data } => reparse_data.read_data(data),
            _ => Ok(()),
        }
    }

    /// Reads a reparse point from a MFT attribute.
    pub fn from_attribute(mft_attribute: &NtfsMftAttribute) -> Result<Self, ErrorTrace> {
        if mft_attribute.attribute_type != NTFS_ATTRIBUTE_TYPE_REPARSE_POINT {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported attribute type: 0x{:08x}",
                mft_attribute.attribute_type
            )));
        }
        if !mft_attribute.is_resident() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported non-resident $REPARSE_POINT attribute"
            ));
        }
        let mediator = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print(format!(
                "NtfsReparsePoint data of size: {} at offset: 0 (0x00000000)\n",
                mft_attribute.resident_data.len(),
            ));
            mediator.debug_print_data(&mft_attribute.resident_data, true);
            mediator.debug_print(NtfsReparsePointHeader::debug_read_data(
                &mft_attribute.resident_data[0..8],
            ));
        }
        let mut reparse_point_header: NtfsReparsePointHeader = NtfsReparsePointHeader::new();
        reparse_point_header.read_data(&mft_attribute.resident_data)?;

        if mediator.debug_output {
            match reparse_point_header.tag {
                0x80000017 => mediator.debug_print(NtfsWofReparseData::debug_read_data(
                    &mft_attribute.resident_data[8..],
                )),
                0xa000000c => mediator.debug_print(NtfsSymoblicLinkReparseData::debug_read_data(
                    &mft_attribute.resident_data[8..],
                )),
                _ => {}
            }
        }
        let mut reparse_point: NtfsReparsePoint = NtfsReparsePoint::new(reparse_point_header.tag);
        reparse_point.read_data(&mft_attribute.resident_data[8..])?;

        Ok(reparse_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_reparse_tag() {
        let test_struct = NtfsReparsePoint::new(0x80000017);

        let reparse_tag: u32 = test_struct.get_reparse_tag();
        assert_eq!(reparse_tag, 0x80000017);
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = NtfsReparsePoint::new(0x80000017);

        let test_data: Vec<u8> = vec![
            0x17, 0x00, 0x00, 0x80, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
        ];
        test_struct.read_data(&test_data[8..])?;

        Ok(())
    }

    // TODO: add tests for from_attribute
}
