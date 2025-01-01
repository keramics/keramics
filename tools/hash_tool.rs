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

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use keramics::formatters::format_as_string;
use keramics::hashes::{
    DigestHashContext, Md5Context, Sha1Context, Sha224Context, Sha256Context, Sha512Context,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DigestHashType {
    /// MD5
    Md5,

    /// SHA1
    Sha1,

    /// SHA-224
    Sha224,

    /// SHA-256
    Sha256,

    /// SHA-512
    Sha512,
}

#[derive(Parser)]
#[command(version, about = "Calculate a digest hash of data", long_about = None)]
struct CommandLineArguments {
    /// Digest hash type
    #[arg(short, long, default_value_t = DigestHashType::Md5, value_enum)]
    digest_hash_type: DigestHashType,

    /// Path of the file to read the data from, if not provided the data will be read from standard input
    source: Option<PathBuf>,
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    let mut reader: Box<dyn BufRead> = match arguments.source {
        None => Box::new(BufReader::new(io::stdin())),
        Some(path) => Box::new(BufReader::new(File::open(path).unwrap())),
    };
    let mut hash_context: Box<dyn DigestHashContext> = match arguments.digest_hash_type {
        DigestHashType::Md5 => Box::new(Md5Context::new()),
        DigestHashType::Sha1 => Box::new(Sha1Context::new()),
        DigestHashType::Sha224 => Box::new(Sha224Context::new()),
        DigestHashType::Sha256 => Box::new(Sha256Context::new()),
        DigestHashType::Sha512 => Box::new(Sha512Context::new()),
    };
    let mut data: [u8; 65536] = [0; 65536];

    while let Ok(read_count) = reader.read(&mut data) {
        if read_count == 0 {
            break;
        }
        hash_context.update(&data[0..read_count]);
    }
    let hash = hash_context.finalize();

    match arguments.digest_hash_type {
        DigestHashType::Md5 => println!("MD5: {}", format_as_string(&hash)),
        DigestHashType::Sha1 => println!("SHA1: {}", format_as_string(&hash)),
        DigestHashType::Sha224 => println!("SHA-224: {}", format_as_string(&hash)),
        DigestHashType::Sha256 => println!("SHA-256: {}", format_as_string(&hash)),
        DigestHashType::Sha512 => println!("SHA-512: {}", format_as_string(&hash)),
    };
    ExitCode::SUCCESS
}
