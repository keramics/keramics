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

use std::rc::Rc;

use libfuzzer_sys::fuzz_target;

use keramics::formats::sparseimage::SparseImageFile;
use keramics::types::SharedValue;
use keramics::vfs::{FakeFileEntry, VfsFileSystem, VfsPath, VfsPathType};

//  Mac OS sparse image (.sparseimage) file fuzz target.
fuzz_target!(|data: &[u8]| {
    let mut fake_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Fake);
    if let VfsFileSystem::Fake(file_system) = &mut fake_file_system {
        let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file(&data);
        _ = file_system.add_file_entry("/input", fake_file_entry);
    }
    let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Fake, "/input", None);
    _ = sparseimage_file.open(&Rc::new(fake_file_system), &vfs_path);
});
