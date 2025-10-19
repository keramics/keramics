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

#![no_main]

use libfuzzer_sys::fuzz_target;

use keramics_core::{DataStreamReference, open_fake_data_stream};
use keramics_formats::ext::ExtFileSystem;

// Extended File System (ext) fuzz target.
fuzz_target!(|data: &[u8]| {
    let mut ext_file_system: ExtFileSystem = ExtFileSystem::new();

    let data_stream: DataStreamReference = open_fake_data_stream(&data);
    _ = ext_file_system.read_data_stream(&data_stream);

    _ = ext_file_system.get_root_directory();
});
