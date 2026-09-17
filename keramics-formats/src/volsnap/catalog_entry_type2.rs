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
use keramics_datetime::{DateTime, Filetime};
use keramics_layout_map::LayoutMap;
use keramics_types::{Uuid, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "entry_type", data_type = "u64"),
        field(name = "size", data_type = "u64"),
        field(name = "store_identifier", data_type = "Uuid"),
        field(name = "unknown1", data_type = "u64"),
        field(name = "unknown2", data_type = "u64", format = "hex"),
        field(name = "creation_time", data_type = "Filetime"),
        field(name = "unknown3", data_type = "[u8; 72]"),
    ),
    methods("debug_read_data")
)]
/// Volume Shadow Snapshot (volsnap) catalog entry type 2.
pub struct VolsnapCatalogEntryType2 {
    /// Size.
    pub size: u64,

    /// Store identifier.
    pub store_identifier: Uuid,

    /// Creation time.
    pub creation_time: DateTime,
}

impl VolsnapCatalogEntryType2 {
    /// Creates a new catalog entry type 2.
    pub fn new() -> Self {
        Self {
            size: 0,
            store_identifier: Uuid::new(),
            creation_time: DateTime::NotSet,
        }
    }

    /// Reads the catalog entry type 2 from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let entry_type: u64 = bytes_to_u64_le!(data, 0);

        if entry_type != 2 {
            return Err(keramics_core::error_trace_new!("Unsupported entry type"));
        }
        self.size = bytes_to_u64_le!(data, 8);
        self.store_identifier = Uuid::from_le_bytes(&data[16..32]);

        let filetime: Filetime = Filetime::from_bytes(&data[48..]);

        self.creation_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0x07, 0x00, 0x00,
            0x00, 0x00, 0x9b, 0x81, 0x17, 0x9f, 0xf9, 0xb0, 0xf1, 0x11, 0x90, 0xdc, 0x7c, 0xed,
            0x8d, 0x4e, 0x4e, 0x79, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x43, 0x9f, 0xad, 0x0e, 0x45, 0xdd, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapCatalogEntryType2::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.size, 133103616);
        assert_eq!(
            test_struct.store_identifier.to_string(),
            "9f17819b-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        assert_eq!(
            test_struct.creation_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x1dd450ead9f43b0,
            })
        );
        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapCatalogEntryType2::new();
        let result = test_struct.read_data(&test_data[0..127]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_entry_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VolsnapCatalogEntryType2::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
