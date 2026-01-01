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

use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use keramics_core::{DataStreamReference, ErrorTrace, FakeDataStream};
use keramics_datetime::DateTime;
use keramics_formats::PathComponent;

use crate::enums::VfsFileType;

/// Fake (or virtual) file entry.
pub struct FakeFileEntry {
    /// Name.
    name: PathComponent,

    /// File type.
    file_type: VfsFileType,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Access time.
    access_time: Option<DateTime>,

    /// Change time.
    change_time: Option<DateTime>,

    /// Creation time.
    creation_time: Option<DateTime>,

    /// Modification time.
    modification_time: Option<DateTime>,

    /// Size.
    size: u64,
}

impl FakeFileEntry {
    /// Creates a new directory.
    pub fn new_directory(name: &str) -> Self {
        let current_time: SystemTime = SystemTime::now();

        Self {
            name: PathComponent::from(name),
            data_stream: None,
            file_type: VfsFileType::Directory,
            access_time: Some(DateTime::FakeTime(current_time.clone())),
            change_time: Some(DateTime::FakeTime(current_time.clone())),
            creation_time: Some(DateTime::FakeTime(current_time.clone())),
            modification_time: Some(DateTime::FakeTime(current_time)),
            size: 0,
        }
    }

    /// Creates a new file.
    pub fn new_file(name: &str, data: &[u8]) -> Self {
        let data_size: u64 = data.len() as u64;
        let data_stream: FakeDataStream = FakeDataStream::new(data, data_size);
        let current_time: SystemTime = SystemTime::now();

        Self {
            name: PathComponent::from(name),
            data_stream: Some(Arc::new(RwLock::new(data_stream))),
            file_type: VfsFileType::File,
            access_time: Some(DateTime::FakeTime(current_time.clone())),
            change_time: Some(DateTime::FakeTime(current_time.clone())),
            creation_time: Some(DateTime::FakeTime(current_time.clone())),
            modification_time: Some(DateTime::FakeTime(current_time)),
            size: data_size,
        }
    }

    /// Creates a new root file entry.
    pub fn new_root() -> Self {
        let current_time: SystemTime = SystemTime::now();

        Self {
            name: PathComponent::Root,
            data_stream: None,
            file_type: VfsFileType::Directory,
            access_time: Some(DateTime::FakeTime(current_time.clone())),
            change_time: Some(DateTime::FakeTime(current_time.clone())),
            creation_time: Some(DateTime::FakeTime(current_time.clone())),
            modification_time: Some(DateTime::FakeTime(current_time)),
            size: 0,
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        self.access_time.as_ref()
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        self.change_time.as_ref()
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        self.creation_time.as_ref()
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.file_type != VfsFileType::File {
            return Ok(None);
        }
        match self.data_stream.as_ref() {
            Some(data_stream) => Ok(Some(data_stream.clone())),
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        self.file_type.clone()
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        self.modification_time.as_ref()
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> &PathComponent {
        &self.name
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        todo!();
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<Arc<FakeFileEntry>, ErrorTrace> {
        todo!();
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        self.name == PathComponent::Root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
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
        ];
    }

    fn get_fake_file_entry() -> FakeFileEntry {
        let test_data: Vec<u8> = get_test_data();

        FakeFileEntry::new_file("file.txt", &test_data)
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = fake_file_entry.get_access_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_change_time() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = fake_file_entry.get_change_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = fake_file_entry.get_creation_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let result: Option<DataStreamReference> = fake_file_entry.get_data_stream()?;
        assert!(result.is_some());

        let mut test_data: Vec<u8> = vec![0; 202];
        let read_count: usize = match result.unwrap().write() {
            Ok(mut data_stream) => data_stream.read(&mut test_data)?,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
        assert_eq!(read_count, 202);

        let expected_data: String = [
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        ]
        .join("");

        assert_eq!(test_data, expected_data.as_bytes());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let file_type: VfsFileType = fake_file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = fake_file_entry.get_modification_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let name: &PathComponent = fake_file_entry.get_name();
        assert_eq!(name, &PathComponent::from("file.txt"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let fake_file_entry: FakeFileEntry = get_fake_file_entry();

        let size: u64 = fake_file_entry.get_size();
        assert_eq!(size, 202);

        Ok(())
    }
}
