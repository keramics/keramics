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

use std::rc::Rc;
use std::sync::Arc;

use crate::types::SharedValue;

use super::file_system::VfsFileSystem;
use super::path::VfsPath;
use super::resolver::VfsResolver;
use super::traits::{VfsDataStream, VfsFileEntry};

pub type VfsDataStreamReference = SharedValue<Box<dyn VfsDataStream>>;

pub type VfsFileEntryReference = Box<dyn VfsFileEntry>;

pub type VfsFileSystemReference = SharedValue<VfsFileSystem>;

pub type VfsPathReference = Rc<VfsPath>;

pub type VfsResolverReference = Arc<VfsResolver>;
