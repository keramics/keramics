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

use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use clap_num::maybe_hex;

use keramics_checksums::{Adler32Context, Crc32Context, Fletcher64Context};
use keramics_compression::{
    AdcContext, Bzip2Context, DeflateContext, LzfseContext, Lznt1Context, LzvnContext, LzxContext,
    LzxpressContext, LzxpressHuffmanContext, ZlibContext,
};
use keramics_core::ErrorTrace;
use keramics_core::formatters::format_as_hexdump;

mod enums;

use crate::enums::{ChecksumType, CompressionType};

#[derive(Parser)]
#[command(version, about = "Multi purpose utility", long_about = None)]
struct CommandLineArguments {
    #[arg(short, long, default_value_t = 0, value_parser=maybe_hex::<u64>)]
    /// Offset within the input file
    offset: u64,

    #[arg(short, long, default_value_t = 0, value_parser=maybe_hex::<u64>)]
    /// Size of the input data
    size: u64,

    /// Path of the input file
    source: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Calculate the checksum of the input data
    Checksum(ChecksumCommandArguments),

    /// Decompress the input data
    Decompress(DecompressCommandArguments),
}

#[derive(Args, Debug)]
struct ChecksumCommandArguments {
    /// Checksum type
    #[arg(value_enum)]
    checksum: ChecksumType,

    #[arg(short, long, default_value_t = 0, value_parser=maybe_hex::<u64>)]
    /// Initial value
    initial_value: u64,

    #[arg(short, long, default_value_t = 0x04c11db7, value_parser=maybe_hex::<u64>)]
    /// Polynomial
    polynomial: u64,
    // TODO: add option to calculate "weak" CRC
}

#[derive(Args, Debug)]
struct DecompressCommandArguments {
    /// Compression type
    #[arg(value_enum)]
    compression: CompressionType,
}

/// Multi purpose tool.
struct UtilTool {}

impl UtilTool {
    /// Calculates a specific checksum of the data.
    fn calculate_checksum(
        checksum: &ChecksumType,
        polynomial: u64,
        initial_value: u64,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        match checksum {
            ChecksumType::Adler32 => {
                let mut adler32_context: Adler32Context = Adler32Context::new(initial_value as u32);
                adler32_context.update(&data);
                let checksum: u32 = adler32_context.finalize();

                println!(
                    "Adler-32 (initial value: 0x{:08x}): 0x{:08x}",
                    initial_value, checksum
                );
            }
            ChecksumType::Crc32 => {
                let mut crc32_context: Crc32Context =
                    Crc32Context::new(polynomial as u32, initial_value as u32);
                crc32_context.update(&data);
                let checksum: u32 = crc32_context.finalize();

                println!(
                    "CRC-32 (polynomial: 0x{:08x}, initial value: 0x{:08x}): 0x{:08x}",
                    polynomial, initial_value, checksum
                );
            }
            ChecksumType::Fletcher64 => {
                let mut fletcher64_context: Fletcher64Context =
                    Fletcher64Context::new(initial_value);
                fletcher64_context.update(&data);
                let checksum: u64 = fletcher64_context.finalize();

                println!(
                    "Fletcher-64 (initial value: 0x{:08x}): 0x{:08x}",
                    initial_value, checksum
                );
            }
        }
        Ok(())
    }

    /// Decompresses data with a specific compression method.
    fn decompress(compression: &CompressionType, compressed_data: &[u8]) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; compressed_data.len()];

        match compression {
            CompressionType::Adc => {
                let mut adc_context: AdcContext = AdcContext::new();

                match adc_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to ADC decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Bzip2 => {
                let mut bzip2_context: Bzip2Context = Bzip2Context::new();

                match bzip2_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to bzip2 decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Deflate => {
                let mut deflate_context: DeflateContext = DeflateContext::new();

                match deflate_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to DEFLATE decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Lzfse => {
                let mut lzfse_context: LzfseContext = LzfseContext::new();

                match lzfse_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to LZFSE decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Lznt1 => {
                let mut lznt1_context: Lznt1Context = Lznt1Context::new();

                match lznt1_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to LZNT1 decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Lzvn => {
                let mut lzvn_context: LzvnContext = LzvnContext::new();

                match lzvn_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to LZVN decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Lzx => {
                let mut lzx_context: LzxContext = LzxContext::new();

                match lzx_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to LZX decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Lzxpress => {
                let mut lzxpress_context: LzxpressContext = LzxpressContext::new();

                match lzxpress_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to LZXPRESS decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::LzxpressHuffman => {
                let mut lzxpress_huffman_context: LzxpressHuffmanContext =
                    LzxpressHuffmanContext::new();

                match lzxpress_huffman_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to LZXPRESS Huffman decompress data"
                        );
                        return Err(error);
                    }
                }
            }
            CompressionType::Zlib => {
                let mut zlib_context: ZlibContext = ZlibContext::new();

                match zlib_context.decompress(&compressed_data, &mut data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to zlib decompress data"
                        );
                        return Err(error);
                    }
                }
            }
        }
        print!("{}", format_as_hexdump(&data, true));

        Ok(())
    }

    /// Reads data from a file.
    fn read_data(path: &PathBuf, offset: u64, data: &mut [u8]) -> Result<(), ErrorTrace> {
        let mut file: File = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to open file",
                    error
                ));
            }
        };
        match file.seek(SeekFrom::Start(offset)) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    format!("Unable to seek offset: {} (0x{:08x})", offset, offset),
                    error
                ));
            }
        }
        match file.read(data) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    format!(
                        "Unable to read data at offset: {} (0x{:08x})",
                        offset, offset
                    ),
                    error
                ));
            }
        }
        Ok(())
    }
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    match arguments.source.to_str() {
        Some(value) => value,
        None => {
            println!("Missing source");
            return ExitCode::FAILURE;
        }
    };
    let mut data_size: u64 = arguments.size;

    if data_size == 0 {
        let metadata: Metadata = match std::fs::metadata(&arguments.source) {
            Ok(metadata) => metadata,
            Err(error) => {
                println!("Unable to determine file metadata\n{}", error);
                return ExitCode::FAILURE;
            }
        };
        data_size = metadata.len();
    }
    // Note that 16777216 is an arbitrary chosen limit.
    if data_size > 16777216 {
        println!("Invalid size value out of bounds");
        return ExitCode::FAILURE;
    }
    match arguments.command {
        Commands::Checksum(command_arguments) => {
            let mut data: Vec<u8> = vec![0; data_size as usize];

            match UtilTool::read_data(&arguments.source, arguments.offset, &mut data) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to read data\n{}", error);
                    return ExitCode::FAILURE;
                }
            }
            match UtilTool::calculate_checksum(
                &command_arguments.checksum,
                command_arguments.polynomial,
                command_arguments.initial_value,
                &data,
            ) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to calculate checksum\n{}", error);
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Decompress(command_arguments) => {
            let mut data: Vec<u8> = vec![0; data_size as usize];

            match UtilTool::read_data(&arguments.source, arguments.offset, &mut data) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to read data\n{}", error);
                    return ExitCode::FAILURE;
                }
            }
            match UtilTool::decompress(&command_arguments.compression, &data) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to decompress\n{}", error);
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x41, 0x20, 0x63, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x79, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x76, 0x61, 0x72, 0x69,
            0x6f, 0x75, 0x73, 0x20, 0x68, 0x61, 0x72, 0x64, 0x2c, 0x20, 0x62, 0x72, 0x69, 0x74,
            0x74, 0x6c, 0x65, 0x2c, 0x20, 0x68, 0x65, 0x61, 0x74, 0x2d, 0x72, 0x65, 0x73, 0x69,
            0x73, 0x74, 0x61, 0x6e, 0x74, 0x2c, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x72,
            0x72, 0x6f, 0x73, 0x69, 0x6f, 0x6e, 0x2d, 0x72, 0x65, 0x73, 0x69, 0x73, 0x74, 0x61,
            0x6e, 0x74, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x73, 0x20, 0x6d,
            0x61, 0x64, 0x65, 0x20, 0x62, 0x79, 0x20, 0x73, 0x68, 0x61, 0x70, 0x69, 0x6e, 0x67,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x6e, 0x20, 0x66, 0x69, 0x72, 0x69,
            0x6e, 0x67, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69,
            0x63, 0x2c, 0x20, 0x6e, 0x6f, 0x6e, 0x6d, 0x65, 0x74, 0x61, 0x6c, 0x6c, 0x69, 0x63,
            0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x2c, 0x20, 0x73, 0x75, 0x63,
            0x68, 0x20, 0x61, 0x73, 0x20, 0x63, 0x6c, 0x61, 0x79, 0x2c, 0x20, 0x61, 0x74, 0x20,
            0x61, 0x20, 0x68, 0x69, 0x67, 0x68, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x65, 0x72, 0x61,
            0x74, 0x75, 0x72, 0x65, 0x2e, 0x0a,
        ]
    }

    #[test]
    fn test_calculate_checksum_with_adler32() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        UtilTool::calculate_checksum(&ChecksumType::Adler32, 0x04c11db7, 0, &test_data)?;

        Ok(())
    }

    #[test]
    fn test_calculate_checksum_with_crc32() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        UtilTool::calculate_checksum(&ChecksumType::Crc32, 0x04c11db7, 0, &test_data)?;

        Ok(())
    }

    #[test]
    fn test_calculate_checksum_with_fletcher64() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        UtilTool::calculate_checksum(&ChecksumType::Fletcher64, 0x04c11db7, 0, &test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let path: PathBuf = PathBuf::from("../test_data/lzvn/compressed");

        let mut data: Vec<u8> = vec![0; 29];
        UtilTool::read_data(&path, 0, &mut data)?;
        assert_eq!(data[0], 0xe0);
        assert_eq!(data[1], 0x03);

        let mut data: Vec<u8> = vec![0; 3];
        UtilTool::read_data(&path, 1, &mut data)?;
        assert_eq!(data[0], 0x03);
        assert_eq!(data[1], 0x4d);
        assert_eq!(data[2], 0x79);

        Ok(())
    }

    #[test]
    fn test_read_data_with_nonexistent_file() {
        let path: PathBuf = PathBuf::from("../test_data/nonexistent_file");

        let mut data: Vec<u8> = vec![0; 1];
        let result: Result<(), ErrorTrace> = UtilTool::read_data(&path, 0, &mut data);

        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_with_adc() -> Result<(), ErrorTrace> {
        let compressed_data: Vec<u8> = match std::fs::read("../test_data/adc/compressed") {
            Ok(data) => data,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to read test data file",
                    error
                ));
            }
        };
        UtilTool::decompress(&CompressionType::Adc, &compressed_data)?;

        Ok(())
    }

    #[test]
    fn test_decompress_with_lzn1() -> Result<(), ErrorTrace> {
        let compressed_data: Vec<u8> = match std::fs::read("../test_data/lznt1/compressed") {
            Ok(data) => data,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to read test data file",
                    error
                ));
            }
        };
        UtilTool::decompress(&CompressionType::Lznt1, &compressed_data)?;

        Ok(())
    }

    #[test]
    fn test_decompress_with_lzvn() -> Result<(), ErrorTrace> {
        let compressed_data: Vec<u8> = match std::fs::read("../test_data/lzvn/compressed") {
            Ok(data) => data,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to read test data file",
                    error
                ));
            }
        };
        UtilTool::decompress(&CompressionType::Lzvn, &compressed_data)?;

        Ok(())
    }

    #[test]
    fn test_decompress_with_lzxpress() -> Result<(), ErrorTrace> {
        let compressed_data: Vec<u8> = match std::fs::read("../test_data/lzxpress/compressed") {
            Ok(data) => data,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to read test data file",
                    error
                ));
            }
        };
        UtilTool::decompress(&CompressionType::Lzxpress, &compressed_data)?;

        Ok(())
    }

    #[test]
    fn test_decompress_with_lzxpress_huffman() -> Result<(), ErrorTrace> {
        let compressed_data: Vec<u8> =
            match std::fs::read("../test_data/lzxpress_huffman/compressed") {
                Ok(data) => data,
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to read test data file",
                        error
                    ));
                }
            };
        UtilTool::decompress(&CompressionType::LzxpressHuffman, &compressed_data)?;

        Ok(())
    }
}
