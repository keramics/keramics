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

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;

use keramics_core::{DataStreamReference, ErrorTrace, open_fake_data_stream};
use keramics_formats::ewf::EwfImage;
use keramics_formats::{FileResolver, FileResolverReference, PathComponent};

pub struct EwfFuzzFileResolver {
    file_name: PathComponent,
    data: Vec<u8>,
}

impl EwfFuzzFileResolver {
    pub fn new(file_name: &str, data: &[u8]) -> Self {
        Self {
            file_name: PathComponent::from(file_name),
            data: data.to_vec(),
        }
    }
}

impl FileResolver for EwfFuzzFileResolver {
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

// Expert Witness Compression Format (EWF) image fuzz target.
fuzz_target!(|data: &[u8]| {
    let mut ewf_image: EwfImage = EwfImage::new();

    let file_resolver: EwfFuzzFileResolver = EwfFuzzFileResolver::new("fuzz.E01", &data);
    let file_resolver_reference: FileResolverReference = Arc::new(Box::new(file_resolver));
    let file_name: PathComponent = PathComponent::from("fuzz.E01");

    _ = ewf_image.open(&file_resolver_reference, &file_name);
});
