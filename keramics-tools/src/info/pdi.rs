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
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::pdi::{
    PdiImage, PdiSegmentDescriptor, PdiSegmentFileDescriptor, PdiSegmentFileType,
    PdiSnapshotDescriptor,
};
use keramics_formats::{FileResolverReference, open_os_file_resolver};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

/// Information about a Parallels Disk Image (PDI) image.
struct PdiImageInfo<'a> {
    /// Image.
    image: &'a PdiImage,
}

impl<'a> PdiImageInfo<'a> {
    /// Creates new image information.
    fn new(image: &'a PdiImage) -> Self {
        Self { image }
    }
}

impl<'a> fmt::Display for PdiImageInfo<'a> {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Parallels Disk Image (PDI) information:")?;

        writeln!(
            formatter,
            "    Number of segments\t\t\t\t: {}",
            self.image.get_number_of_segments()
        )?;
        writeln!(
            formatter,
            "    Number of snapshots\t\t\t\t: {}",
            self.image.get_number_of_snapshots()
        )?;
        writeln!(formatter)?;

        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.image.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {}",
            self.image.get_bytes_per_sector()
        )?;

        // TODO: print additional information

        writeln!(formatter)
    }
}

/// Information about a Parallels Disk Image (PDI) snapshot.
struct PdiSnapshotInfo<'a> {
    /// Snapshot index.
    index: usize,

    /// Snapshot.
    snapshot: &'a PdiSnapshotDescriptor,
}

impl<'a> PdiSnapshotInfo<'a> {
    /// Creates new snapshot information.
    fn new(index: usize, snapshot: &'a PdiSnapshotDescriptor) -> Self {
        Self { index, snapshot }
    }
}

impl<'a> fmt::Display for PdiSnapshotInfo<'a> {
    /// Formats snapshot information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Snapshot: {}", self.index + 1)?;

        writeln!(
            formatter,
            "        Identifier\t\t\t\t: {}",
            self.snapshot.get_identifier()
        )?;
        if let Some(parent_identifier) = self.snapshot.get_parent_identifier() {
            writeln!(
                formatter,
                "        Parent identifier\t\t\t: {}",
                parent_identifier
            )?;
        }
        writeln!(formatter)
    }
}

/// Information about a Parallels Disk Image (PDI) image.
pub struct PdiInfo {}

impl PdiInfo {
    /// Opens an image.
    fn open_image(path_buf: &PathBuf) -> Result<PdiImage, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let mut pdi_image: PdiImage = PdiImage::new();

        match pdi_image.open(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open PDI image");
                return Err(error);
            }
        }
        Ok(pdi_image)
    }

    /// Prints information about an image.
    pub fn print_image(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let pdi_image: PdiImage = match Self::open_image(path_buf) {
            Ok(pdi_image) => pdi_image,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        };
        let image_information: PdiImageInfo = PdiImageInfo::new(&pdi_image);

        print!("{}", image_information);

        for segment_index in 0..pdi_image.get_number_of_segments() {
            // TODO: refactor into SegmentInfo
            let segment: &PdiSegmentDescriptor = match pdi_image.get_segment_by_index(segment_index)
            {
                Some(segment) => segment,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing segment: {}",
                        segment_index + 1
                    )));
                }
            };
            println!("    Segment: {}", segment_index + 1);

            let byte_size: ByteSize = ByteSize::new(segment.get_size(), 1024);
            println!("        Size\t\t\t\t\t: {}", byte_size);

            let number_of_files: usize = segment.get_number_of_files();
            println!("        Number of files\t\t\t\t: {}", number_of_files);

            println!();

            for file_index in 0..number_of_files {
                let segment_file: &PdiSegmentFileDescriptor =
                    match segment.get_file_by_index(file_index) {
                        Some(segment_file) => segment_file,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing segment: {} file: {}",
                                segment_index + 1,
                                file_index + 1
                            )));
                        }
                    };
                println!("        Segment file: {}", file_index + 1);

                let snapshot_identifier: &Uuid = segment_file.get_snapshot_identifier();
                println!(
                    "            Snapshot identifier\t\t\t: {}",
                    snapshot_identifier
                );

                println!("            Path\t\t\t\t: {}", segment_file.get_path());

                let file_type_string: &str = match segment_file.get_file_type() {
                    &PdiSegmentFileType::Compressed => "Compressed",
                    &PdiSegmentFileType::Plain => "Plain",
                    _ => "Unknown",
                };
                println!("            Type\t\t\t\t: {}", file_type_string);

                println!();
            }
        }
        for snapshot_index in 0..pdi_image.get_number_of_snapshots() {
            let pdi_snapshot: &PdiSnapshotDescriptor =
                match pdi_image.get_snapshot_by_index(snapshot_index) {
                    Some(snapshot) => snapshot,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing snapshot: {}",
                            snapshot_index + 1
                        )));
                    }
                };
            let snapshot_information: PdiSnapshotInfo =
                PdiSnapshotInfo::new(snapshot_index, &pdi_snapshot);

            print!("{}", snapshot_information);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::assert_lines_eq;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/pdi/hfsplus.hdd/DiskDescriptor.xml");
        let pdi_image: PdiImage = PdiInfo::open_image(&path_buf)?;
        let test_struct: PdiImageInfo = PdiImageInfo::new(&pdi_image);

        let expected_string: &str = concat!(
            "Parallels Disk Image (PDI) information:\n",
            "    Number of segments\t\t\t\t: 1\n",
            "    Number of snapshots\t\t\t\t: 1\n",
            "\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 32.0 MiB (33554432 bytes)\n",
            "        Bytes per sector\t\t\t: 512\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for PdiSnapshotInfo

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
