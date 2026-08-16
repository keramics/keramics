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
use keramics_types::bytes_to_u16_le;

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "head", data_type = "u8"),
        field(name = "cylinder", data_type = "BitField16<6>"),
        field(name = "sector", data_type = "BitField16<10>"),
    ),
    methods("debug_read_data")
)]
/// Cylinder Head Sector (CHS) address.
pub struct MbrChsAddress {
    /// Head.
    pub head: u8,

    /// Cylinder.
    pub cylinder: u16,

    /// Sector.
    pub sector: u8,
}

impl MbrChsAddress {
    /// Creates a new CHS address.
    pub fn new() -> Self {
        Self {
            head: 0,
            cylinder: 0,
            sector: 0,
        }
    }

    /// Calculates the LBA from the CHS address.
    pub fn calculate_lba(&self, heads_per_cylinder: u32, sectors_per_track: u32) -> u32 {
        ((((self.cylinder as u32) * heads_per_cylinder) + (self.head as u32)) * sectors_per_track)
            + ((self.sector as u32) - 1)
    }

    /// Reads the CHS address from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 3 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let value_16bit: u16 = bytes_to_u16_le!(data, 1);

        self.head = data[0];
        self.cylinder = value_16bit >> 6;
        self.sector = (value_16bit & 0x003f) as u8;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![0x02, 0xc1, 0x00];
    }

    #[test]
    fn test_calculate_lba() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = MbrChsAddress::new();
        test_struct.read_data(&test_data)?;

        let lba: u32 = test_struct.calculate_lba(255, 63);
        assert_eq!(lba, 48321);

        Ok(())
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = MbrChsAddress::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.head, 2);
        assert_eq!(test_struct.cylinder, 3);
        assert_eq!(test_struct.sector, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = MbrChsAddress::new();
        let result = test_struct.read_data(&test_data[0..2]);
        assert!(result.is_err());
    }
}
