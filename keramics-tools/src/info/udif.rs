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

use std::collections::HashMap;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::udif::{UdifCompressionMethod, UdifFile};

use crate::formatters::format_as_bytesize;

/// Information about an Universal Disk Image Format (UDIF) file.
pub struct UdifInfo {}

impl UdifInfo {
    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut udif_file: UdifFile = UdifFile::new();

        match udif_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open UDIF file");
                return Err(error);
            }
        };
        let compression_methods = HashMap::<UdifCompressionMethod, &'static str>::from([
            (UdifCompressionMethod::Adc, "ADC"),
            (UdifCompressionMethod::Bzip2, "bzip2"),
            (UdifCompressionMethod::Lzfse, "LZFSE/LZVN"),
            (UdifCompressionMethod::Lzma, "LZMA"),
            (UdifCompressionMethod::None, "Uncompressed"),
            (UdifCompressionMethod::Zlib, "zlib"),
        ]);

        let compression_method_string: &str = compression_methods
            .get(&udif_file.compression_method)
            .unwrap();

        println!("Universal Disk Image Format (UDIF) information:");

        if udif_file.media_size < 1024 {
            println!("    Media size\t\t\t\t: {} bytes", udif_file.media_size);
        } else {
            let media_size_string: String = format_as_bytesize(udif_file.media_size, 1024);
            println!(
                "    Media size\t\t\t\t: {} ({} bytes)",
                media_size_string, udif_file.media_size
            );
        }
        println!(
            "    Bytes per sector\t\t\t: {} bytes",
            udif_file.bytes_per_sector
        );
        println!(
            "    Compression method\t\t\t: {}",
            compression_method_string
        );

        println!("");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for print_file
}
