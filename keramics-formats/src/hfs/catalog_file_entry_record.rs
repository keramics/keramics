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

use super::catalog_file_record::HfsCatalogFileRecord;
use super::catalog_folder_record::HfsCatalogFolderRecord;

#[derive(Clone)]
pub enum HfsCatalogFileEntryRecord {
    File(HfsCatalogFileRecord),
    Folder(HfsCatalogFolderRecord),
}

impl HfsCatalogFileEntryRecord {
    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> u32 {
        match self {
            HfsCatalogFileEntryRecord::File(file_record) => file_record.identifier,
            HfsCatalogFileEntryRecord::Folder(folder_record) => folder_record.identifier,
        }
    }
}
