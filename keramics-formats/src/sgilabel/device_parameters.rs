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
use keramics_layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "skew", data_type = "u8"),
        field(name = "gap1_size", data_type = "u8"),
        field(name = "gap2_size", data_type = "u8"),
        field(name = "number_of_spare_cylinders", data_type = "u8"),
        field(name = "number_of_cylinders", data_type = "u16"),
        field(name = "heads_per_volume", data_type = "u16"),
        field(name = "tracks_per_cylinders", data_type = "u16"),
        field(name = "unknown1", data_type = "u8"),
        field(name = "unknown2", data_type = "[u8; 3]"),
        field(name = "sectors_per_track", data_type = "u16"),
        field(name = "bytes_per_sector", data_type = "u16"),
        field(name = "unknown3", data_type = "[u8; 30]"),
    ),
    methods("debug_read_data")
)]
/// SGI disklabel (sgilabel) device parameters.
pub struct SgiDeviceParameters {}

impl SgiDeviceParameters {
    /// Creates a new device parameters.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the device parameters from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 48 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x3f, 0x02, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = SgiDeviceParameters::new();
        test_struct.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = SgiDeviceParameters::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }
}
