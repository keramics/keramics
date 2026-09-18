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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::volsnap::{VolsnapBackingVolume, VolsnapSnapshot};

use crate::formatters::ByteSize;

use super::windows::FiletimeDateTimeInfo;

/// Volume Shadow Snapshot (volsnap) backing volume information.
struct VolsnapBackingVolumeInfo<'a> {
    /// Backing volume.
    backing_volume: &'a VolsnapBackingVolume,
}

impl<'a> VolsnapBackingVolumeInfo<'a> {
    /// Creates new backing volume information.
    fn new(backing_volume: &'a VolsnapBackingVolume) -> Self {
        Self { backing_volume }
    }
}

impl<'a> fmt::Display for VolsnapBackingVolumeInfo<'a> {
    /// Formats backing volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Volume Shadow Snapshot (volsnap) information:")?;

        writeln!(
            formatter,
            "    Volume identifier\t\t\t\t: {}",
            self.backing_volume.get_volume_identifier()
        )?;
        writeln!(
            formatter,
            "    Storage volume identifier\t\t: {}",
            self.backing_volume.get_storage_volume_identifier()
        )?;
        writeln!(
            formatter,
            "    Number of shadow copies\t\t\t: {}",
            self.backing_volume.get_number_of_snapshots(),
        )?;
        writeln!(formatter)
    }
}

/// Volume Shadow Snapshot (volsnap) snapshot information.
struct VolsnapSnapshotInfo<'a> {
    /// Snapshot index.
    snapshot_index: usize,

    /// Snapshot.
    snapshot: &'a VolsnapSnapshot,
}

impl<'a> VolsnapSnapshotInfo<'a> {
    /// Creates new snapshot information.
    fn new(snapshot_index: usize, snapshot: &'a VolsnapSnapshot) -> Self {
        Self {
            snapshot_index,
            snapshot,
        }
    }
}

impl<'a> fmt::Display for VolsnapSnapshotInfo<'a> {
    /// Formats snapshot information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Shadow copy: {}:", self.snapshot_index + 1)?;

        if let Some(copy_identifier) = self.snapshot.get_copy_identifier() {
            writeln!(
                formatter,
                "        Copy identifier\t\t\t\t: {}",
                copy_identifier
            )?;
        }
        if let Some(copy_set_identifier) = self.snapshot.get_copy_set_identifier() {
            writeln!(
                formatter,
                "        Copy set identifier\t\t\t: {}",
                copy_set_identifier
            )?;
        }
        writeln!(
            formatter,
            "        Store identifier\t\t\t: {}",
            self.snapshot.get_store_identifier()
        )?;
        // TODO: print copy set identifier

        if let Some(creation_time) = self.snapshot.get_creation_time() {
            let date_time_info: FiletimeDateTimeInfo = FiletimeDateTimeInfo::new(creation_time);
            writeln!(
                formatter,
                "        Creation time\t\t\t\t: {}",
                date_time_info
            )?;
        }
        if let Some(size) = self.snapshot.get_size() {
            let byte_size: ByteSize = ByteSize::new(*size, 1024);
            writeln!(formatter, "        Size\t\t\t\t\t: {}", byte_size)?;
        }
        // TODO: print copy identifier

        writeln!(formatter)
    }
}

/// Information about Volume Shadow Snapshot (volsnap).
pub struct VolsnapInfo {}

impl VolsnapInfo {
    /// Opens a backing volume.
    pub fn open_backing_volume(
        data_stream: &DataStreamReference,
    ) -> Result<VolsnapBackingVolume, ErrorTrace> {
        let mut volsnap_backing_volume: VolsnapBackingVolume = VolsnapBackingVolume::new();

        match volsnap_backing_volume.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open volsnap backing volume."
                );
                return Err(error);
            }
        }
        Ok(volsnap_backing_volume)
    }

    /// Prints information about a backing volume.
    pub fn print_backing_volume(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let volsnap_backing_volume: VolsnapBackingVolume =
            match Self::open_backing_volume(data_stream) {
                Ok(volsnap_backing_volume) => volsnap_backing_volume,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open backing volume");
                    return Err(error);
                }
            };
        let backing_volume_info: VolsnapBackingVolumeInfo =
            VolsnapBackingVolumeInfo::new(&volsnap_backing_volume);

        print!("{}", backing_volume_info);

        for (snapshot_index, result) in volsnap_backing_volume.snapshots().enumerate() {
            let volsnap_snapshot: VolsnapSnapshot = match result {
                Ok(volsnap_snapshot) => volsnap_snapshot,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve snapshot: {}", snapshot_index)
                    );
                    return Err(error);
                }
            };
            let snapshot_info: VolsnapSnapshotInfo =
                VolsnapSnapshotInfo::new(snapshot_index, &volsnap_snapshot);

            print!("{}", snapshot_info);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use keramics_core::open_os_data_stream;
    use keramics_formats::RangeStream;
    use keramics_formats::vhd::VhdFile;

    use crate::assert_lines_eq;

    #[test]
    fn test_backing_volume_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/volsnap/volsnap.vhd");
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            133103616,
        )));
        let volsnap_backing_volume: VolsnapBackingVolume =
            VolsnapInfo::open_backing_volume(&data_stream)?;

        let test_struct: VolsnapBackingVolumeInfo =
            VolsnapBackingVolumeInfo::new(&volsnap_backing_volume);

        let expected_string: &str = concat!(
            "Volume Shadow Snapshot (volsnap) information:\n",
            "    Volume identifier\t\t\t\t: 9f178190-b0f9-11f1-90dc-7ced8d4e4e79\n",
            "    Storage volume identifier\t\t: 9f178190-b0f9-11f1-90dc-7ced8d4e4e79\n",
            "    Number of shadow copies\t\t\t: 2\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_snapshot_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/volsnap/volsnap.vhd");
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            133103616,
        )));
        let volsnap_backing_volume: VolsnapBackingVolume =
            VolsnapInfo::open_backing_volume(&data_stream)?;
        let volsnap_snapshot: VolsnapSnapshot = volsnap_backing_volume.get_snapshot_by_index(0)?;

        let test_struct: VolsnapSnapshotInfo = VolsnapSnapshotInfo::new(0, &volsnap_snapshot);

        let expected_string: &str = concat!(
            "    Shadow copy: 1:\n",
            "        Copy identifier\t\t\t\t: 54ab4fc9-3eae-4ded-8e85-6f1f864251dc\n",
            "        Copy set identifier\t\t\t: 6755c5a0-eb62-41e9-91d6-490246c61375\n",
            "        Store identifier\t\t\t: 9f17819b-b0f9-11f1-90dc-7ced8d4e4e79\n",
            "        Creation time\t\t\t\t: 2026-09-15T12:35:23.5737520+00:00\n",
            "        Size\t\t\t\t\t: 126.9 MiB (133103616 bytes)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_backing_volume
    // TODO: add tests for print_backing_volume
}
