/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::convert::AsRef;
use std::io;
use std::io::Cursor;

use crate::vfs::traits::VfsDataStream;

impl<T: AsRef<[u8]>> VfsDataStream for Cursor<T> {}

// TODO: consider moving this to a separate file.
use crate::types::SharedValue;
use crate::vfs::types::VfsDataStreamReference;

pub fn new_fake_data_stream(data: Vec<u8>) -> io::Result<VfsDataStreamReference> {
    Ok(SharedValue::new(Box::new(Cursor::new(data))))
}
