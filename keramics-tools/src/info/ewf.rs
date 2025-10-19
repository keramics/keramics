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
use std::path::PathBuf;
use std::process::ExitCode;

use keramics_core::formatters::format_as_string;
use keramics_formats::ewf::{EwfHeaderValueType, EwfImage, EwfMediaType};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

use crate::formatters::format_as_bytesize;

/// Prints information about an EWF image.
pub fn print_ewf_image(path: &PathBuf) -> ExitCode {
    let mut base_path: PathBuf = path.clone();
    base_path.pop();

    let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
        Ok(file_resolver) => file_resolver,
        Err(error) => {
            println!("Unable to create file resolver with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let mut ewf_image: EwfImage = EwfImage::new();

    let file_name: PathComponent = match path.file_name() {
        Some(file_name) => match file_name.to_str() {
            Some(file_name) => PathComponent::from(file_name),
            None => {
                println!("Invalid file name");
                return ExitCode::FAILURE;
            }
        },
        None => {
            println!("Missing file name");
            return ExitCode::FAILURE;
        }
    };
    match ewf_image.open(&file_resolver, &file_name) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open EWF image with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    println!("Expert Witness Compression Format (EWF) information:");
    // File format
    if !ewf_image.set_identifier.is_nil() {
        println!(
            "    Set identifier\t\t\t\t: {}",
            ewf_image.set_identifier.to_string(),
        );
    }
    println!(
        "    Sectors per chunk\t\t\t\t: {}",
        ewf_image.sectors_per_chunk
    );
    println!(
        "    Error granularity\t\t\t\t: {} sectors",
        ewf_image.error_granularity
    );
    // Compression method

    println!("");

    println!("    Media information:");

    let media_types = HashMap::<EwfMediaType, &'static str>::from([
        (EwfMediaType::FixedDisk, "fixed disk"),
        (EwfMediaType::LogicalEvidence, "logical evidence"),
        (EwfMediaType::OpticalDisk, "optical disk (CD/DVD/BD)"),
        (EwfMediaType::RemoveableDisk, "removable disk"),
        (EwfMediaType::Unknown, "Unknown"),
    ]);
    let media_type_string: &str = media_types.get(&ewf_image.media_type).unwrap();

    // Media type (combine with Is physical)
    println!("        Media type\t\t\t\t: {}", media_type_string,);
    println!(
        "        Bytes per sector\t\t\t: {}",
        ewf_image.bytes_per_sector
    );
    println!(
        "        Number of sectors\t\t\t: {}",
        ewf_image.number_of_sectors
    );

    if ewf_image.media_size < 1024 {
        println!("        Media size\t\t\t\t: {} bytes", ewf_image.media_size);
    } else {
        let media_size_string: String = format_as_bytesize(ewf_image.media_size, 1024);
        println!(
            "        Media size\t\t\t\t: {} ({} bytes)",
            media_size_string, ewf_image.media_size
        );
    }
    if ewf_image.md5_hash != [0; 16] {
        let hash_string: String = format_as_string(&ewf_image.md5_hash);
        println!("        MD5\t\t\t\t\t: {}", hash_string);
    }
    if ewf_image.sha1_hash != [0; 20] {
        let hash_string: String = format_as_string(&ewf_image.sha1_hash);
        println!("        SHA1\t\t\t\t\t: {}", hash_string);
    }
    println!("");

    println!("    Case information:");

    let header_values: [(EwfHeaderValueType, &str); 15] = [
        (EwfHeaderValueType::CaseNumber, "Case number"),
        (EwfHeaderValueType::Description, "Description"),
        (EwfHeaderValueType::ExaminerName, "Examiner name"),
        (EwfHeaderValueType::EvidenceNumber, "Evidence number"),
        (EwfHeaderValueType::Notes, "Notes"),
        (EwfHeaderValueType::AcquisitionDate, "Acquisition date"),
        (EwfHeaderValueType::SystemDate, "System date"),
        (EwfHeaderValueType::Platform, "Operating system used"),
        (EwfHeaderValueType::Version, "Software version used"),
        (EwfHeaderValueType::PasswordHash, "Password"),
        (EwfHeaderValueType::CompressionLevel, "Compression level"),
        (EwfHeaderValueType::Model, "Model"),
        (EwfHeaderValueType::SerialNumber, "Serial number"),
        (EwfHeaderValueType::DeviceLabel, "Device label"),
        (EwfHeaderValueType::ProcessIdentifier, "Process identifier"),
    ];
    for (header_value_type, description) in header_values {
        match ewf_image.get_header_value(&header_value_type) {
            Some(header_value) => {
                let header_value_string: String = header_value.to_string();

                if header_value_string.is_empty() {
                    continue;
                }
                match &header_value_type {
                    EwfHeaderValueType::PasswordHash => {
                        if header_value_string == "0" {
                            continue;
                        }
                    }
                    _ => {}
                }
                println!(
                    "        {}{}: {}",
                    description,
                    "\t".repeat((40 - description.len()).div_ceil(8)),
                    header_value_string,
                );
            }
            None => {}
        }
    }
    // TODO: print case information
    //
    // TODO: print optical disk session information
    // TODO: print error information

    println!("");

    ExitCode::SUCCESS
}
