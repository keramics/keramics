/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

#[cfg(feature = "debug-trace")]
use keramics_core::DebugTrace;

use super::constants::*;
use super::junction_reparse_data::NtfsJunctionReparseData;
use super::mft_attribute::NtfsMftAttribute;
use super::reparse_point_header::NtfsReparsePointHeader;
use super::symbolic_link_reparse_data::NtfsSymbolicLinkReparseData;
use super::wof_reparse_data::NtfsWofReparseData;

/// New Technologies File System (NTFS) reparse point.
pub enum NtfsReparsePoint {
    Junction {
        reparse_data: NtfsJunctionReparseData,
    },
    SymbolicLink {
        reparse_data: NtfsSymbolicLinkReparseData,
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
            0xa0000003 => NtfsReparsePoint::Junction {
                reparse_data: NtfsJunctionReparseData::new(),
            },
            0xa000000c => NtfsReparsePoint::SymbolicLink {
                reparse_data: NtfsSymbolicLinkReparseData::new(),
            },
            _ => NtfsReparsePoint::Undefined { tag },
        }
    }

    /// Retrieves the reparse tag.
    pub fn get_reparse_tag(&self) -> u32 {
        match self {
            NtfsReparsePoint::Junction { .. } => 0xa0000003,
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
        keramics_core::debug_trace_data_and_structure!(
            "NtfsReparsePoint",
            0,
            &mft_attribute.resident_data,
            mft_attribute.resident_data.len(),
            NtfsReparsePointHeader::debug_read_data(&mft_attribute.resident_data)
        );
        let mut reparse_point_header: NtfsReparsePointHeader = NtfsReparsePointHeader::new();

        match reparse_point_header.read_data(&mft_attribute.resident_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read reparse point header");
                return Err(error);
            }
        }
        #[cfg(feature = "debug-trace")]
        DebugTrace::static_scope(|debug_trace| match reparse_point_header.tag {
            0x80000017 => debug_trace.print_structure(
                NtfsWofReparseData::debug_read_data,
                &mft_attribute.resident_data[8..],
            ),
            0xa0000003 => debug_trace.print_structure(
                NtfsJunctionReparseData::debug_read_data,
                &mft_attribute.resident_data[8..],
            ),
            0xa000000c => debug_trace.print_structure(
                NtfsSymbolicLinkReparseData::debug_read_data,
                &mft_attribute.resident_data[8..],
            ),
            _ => {}
        });
        let mut reparse_point: NtfsReparsePoint = NtfsReparsePoint::new(reparse_point_header.tag);

        match reparse_point.read_data(&mft_attribute.resident_data[8..]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read reparse point data");
                return Err(error);
            }
        }
        Ok(reparse_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_reparse_tag() {
        let test_struct: NtfsReparsePoint = NtfsReparsePoint::new(0x80000017);
        assert_eq!(test_struct.get_reparse_tag(), 0x80000017);

        let junction: NtfsReparsePoint = NtfsReparsePoint::new(0xa0000003);
        assert_eq!(junction.get_reparse_tag(), 0xa0000003);

        let symbolic_link: NtfsReparsePoint = NtfsReparsePoint::new(0xa000000c);
        assert_eq!(symbolic_link.get_reparse_tag(), 0xa000000c);

        let undefined: NtfsReparsePoint = NtfsReparsePoint::new(0x12345678);
        assert_eq!(undefined.get_reparse_tag(), 0x12345678);
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct: NtfsReparsePoint = NtfsReparsePoint::new(0x80000017);

        let test_data: Vec<u8> = vec![
            0x17, 0x00, 0x00, 0x80, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
        ];
        test_struct.read_data(&test_data[8..])?;

        assert!(matches!(
            test_struct,
            NtfsReparsePoint::WindowsOverlayFilter { .. }
        ));

        Ok(())
    }

    #[test]
    fn test_from_attribute() -> Result<(), ErrorTrace> {
        let mut mft_attribute: NtfsMftAttribute = NtfsMftAttribute::new();

        let test_data: Vec<u8> = vec![
            0xc0, 0x00, 0x00, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x3c, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0xa0,
            0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x1a, 0x00, 0x10, 0x00, 0x5c, 0x00,
            0x3f, 0x00, 0x3f, 0x00, 0x5c, 0x00, 0x43, 0x00, 0x3a, 0x00, 0x5c, 0x00, 0x55, 0x00,
            0x73, 0x00, 0x65, 0x00, 0x72, 0x00, 0x73, 0x00, 0x00, 0x00, 0x43, 0x00, 0x3a, 0x00,
            0x5c, 0x00, 0x55, 0x00, 0x73, 0x00, 0x65, 0x00, 0x72, 0x00, 0x73, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        mft_attribute.read_data(&test_data)?;

        let test_struct: NtfsReparsePoint = NtfsReparsePoint::from_attribute(&mft_attribute)?;

        assert!(matches!(test_struct, NtfsReparsePoint::Junction { .. }));

        // Test WindowsOverlayFilter reparse point from attribute.
        let mut overlay_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        overlay_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_REPARSE_POINT;
        overlay_attribute.resident_data = vec![
            0x17, 0x00, 0x00, 0x80, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
        ];
        let overlay_point: NtfsReparsePoint = NtfsReparsePoint::from_attribute(&overlay_attribute)?;
        assert!(matches!(
            overlay_point,
            NtfsReparsePoint::WindowsOverlayFilter { .. }
        ));

        // Test SymbolicLink reparse point from attribute.
        let mut symbolic_link_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        symbolic_link_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_REPARSE_POINT;
        symbolic_link_attribute.resident_data = vec![
            0x0c, 0x00, 0x00, 0xa0, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, b'a', 0x00, b'b', 0x00,
        ];
        let symbolic_link_point: NtfsReparsePoint =
            NtfsReparsePoint::from_attribute(&symbolic_link_attribute)?;
        assert!(matches!(
            symbolic_link_point,
            NtfsReparsePoint::SymbolicLink { .. }
        ));

        // Test Undefined reparse point from attribute.
        let mut undefined_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        undefined_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_REPARSE_POINT;
        undefined_attribute.resident_data = vec![0x78, 0x56, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00];
        let undefined_point: NtfsReparsePoint =
            NtfsReparsePoint::from_attribute(&undefined_attribute)?;
        assert_eq!(undefined_point.get_reparse_tag(), 0x12345678);

        // Test unsupported attribute type error.
        let mut wrong_type_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        wrong_type_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_DATA;
        let result: Result<NtfsReparsePoint, ErrorTrace> =
            NtfsReparsePoint::from_attribute(&wrong_type_attribute);
        assert!(result.is_err());

        // Test non-resident attribute error.
        let mut non_resident_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        non_resident_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_REPARSE_POINT;
        non_resident_attribute.non_resident_flag = 1;
        let result: Result<NtfsReparsePoint, ErrorTrace> =
            NtfsReparsePoint::from_attribute(&non_resident_attribute);
        assert!(result.is_err());

        // Test truncated header error.
        let mut truncated_header_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        truncated_header_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_REPARSE_POINT;
        truncated_header_attribute.resident_data = vec![0; 4];
        let result: Result<NtfsReparsePoint, ErrorTrace> =
            NtfsReparsePoint::from_attribute(&truncated_header_attribute);
        assert!(result.is_err());

        // Test invalid reparse point data error.
        let mut invalid_data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        invalid_data_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_REPARSE_POINT;
        invalid_data_attribute.resident_data =
            vec![0x17, 0x00, 0x00, 0x80, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result: Result<NtfsReparsePoint, ErrorTrace> =
            NtfsReparsePoint::from_attribute(&invalid_data_attribute);
        assert!(result.is_err());

        Ok(())
    }
}
