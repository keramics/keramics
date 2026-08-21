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

#![no_main]

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;

use keramics_core::{DataStreamReference, ErrorTrace, open_fake_data_stream};
use keramics_formats::linuxlvm::{LinuxLvmDataFileDescriptor, LinuxLvmVolumeSystem};
use keramics_formats::{FileResolver, FileResolverReference, PathComponent};

pub struct LinuxLvmFuzzFileResolver {
    file_name: PathComponent,
    data: Vec<u8>,
}

impl LinuxLvmFuzzFileResolver {
    pub fn new(file_name: &str, data: &[u8]) -> Self {
        Self {
            file_name: PathComponent::from(file_name),
            data: data.to_vec(),
        }
    }
}

impl FileResolver for LinuxLvmFuzzFileResolver {
    fn get_data_stream(
        &self,
        path_components: &[PathComponent],
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if path_components[0] == self.file_name {
            let data_stream: DataStreamReference = open_fake_data_stream(&self.data);

            Ok(Some(data_stream))
        } else {
            Ok(None)
        }
    }
}

// Linux Logical Volume Manager (LVM) volume system fuzz target.
fuzz_target!(|data: &[u8]| {
    let mut lvm_volume_system: LinuxLvmVolumeSystem = LinuxLvmVolumeSystem::new();

    let file_resolver: LinuxLvmFuzzFileResolver = LinuxLvmFuzzFileResolver::new("lvm.raw", &data);
    let file_resolver_reference: FileResolverReference = Arc::new(Box::new(file_resolver));

    let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] = [LinuxLvmDataFileDescriptor::new(
        PathComponent::from("lvm.raw"),
        0,
    )];
    _ = lvm_volume_system.open(&file_resolver_reference, &data_file_descriptors);
});
