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

use std::io;

use super::data_run::{NtfsDataRun, NtfsDataRunType};

/// New Technologies File System (NTFS) cluster group.
pub struct NtfsClusterGroup {
    /// First virtual cluster number (VCN).
    pub first_vcn: u64,

    /// Last virtual cluster number (VCN).
    pub last_vcn: u64,

    /// Data runs of the physical extents.
    pub data_runs: Vec<NtfsDataRun>,
}

impl NtfsClusterGroup {
    /// Creates a new cluster group.
    pub fn new(first_vcn: u64, last_vcn: u64) -> Self {
        Self {
            first_vcn: first_vcn,
            last_vcn: last_vcn,
            data_runs: Vec::new(),
        }
    }

    /// Reads the data runs from a buffer.
    pub fn read_data_runs(&mut self, data: &[u8], data_runs_offset: usize) -> io::Result<usize> {
        let mut data_offset: usize = data_runs_offset;
        let data_size: usize = data.len();
        let mut last_block_number: u64 = 0;

        while data_offset < data_size {
            let mut data_run: NtfsDataRun = NtfsDataRun::new();

            let read_count: usize = data_run.read_data(&data[data_offset..], last_block_number)?;
            data_offset += read_count;

            match &data_run.run_type {
                NtfsDataRunType::EndOfList => break,
                NtfsDataRunType::InFile => {
                    last_block_number = data_run.block_number;
                }
                NtfsDataRunType::Sparse => {}
            };
            self.data_runs.push(data_run);
        }
        Ok(data_offset - data_runs_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_data_runs() -> io::Result<()> {
        let data: Vec<u8> = vec![0x11, 0x03, 0x37, 0x01, 0x0d, 0x00];

        let mut cluster_group = NtfsClusterGroup::new(0, 16);

        assert_eq!(cluster_group.first_vcn, 0);
        assert_eq!(cluster_group.last_vcn, 16);
        assert_eq!(cluster_group.data_runs.len(), 0);

        cluster_group.read_data_runs(&data, 0)?;

        assert_eq!(cluster_group.data_runs.len(), 2);

        let data_run: &NtfsDataRun = &cluster_group.data_runs[0];

        assert_eq!(data_run.block_number, 55);
        assert_eq!(data_run.number_of_blocks, 3);
        assert_eq!(data_run.run_type, NtfsDataRunType::InFile);

        let data_run: &NtfsDataRun = &cluster_group.data_runs[1];

        assert_eq!(data_run.block_number, 0);
        assert_eq!(data_run.number_of_blocks, 13);
        assert_eq!(data_run.run_type, NtfsDataRunType::Sparse);

        Ok(())
    }
}
