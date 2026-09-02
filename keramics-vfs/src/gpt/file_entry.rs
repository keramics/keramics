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

use keramics_core::DataStreamReference;
use keramics_formats::gpt::{GptPartition, GptVolumeSystem};
use keramics_types::Uuid;

use crate::partition::{VfsPartitionFileEntry, VfsPartitionIdentifier};
use crate::traits::VfsPartition;

/// GUID Partition Table (GPT) file entry.
pub type GptFileEntry = VfsPartitionFileEntry<GptPartition, GptVolumeSystem>;

impl VfsPartition for GptPartition {
    /// Name prefix.
    const NAME_PREFIX: &'static str = "gpt";

    /// Retrieves the default data stream.
    fn get_data_stream(&self) -> DataStreamReference {
        GptPartition::get_data_stream(self)
    }

    /// Retrieves the partition identifier.
    fn get_identifier(&self) -> Option<VfsPartitionIdentifier> {
        let identifier: &Uuid = GptPartition::get_identifier(self);

        Some(VfsPartitionIdentifier::Uuid(identifier.clone()))
    }

    /// Retrieves the partition number.
    fn get_partition_number(&self) -> usize {
        let gpt_partition_index: usize = GptPartition::get_partition_index(self);

        (gpt_partition_index as usize) + 1
    }

    /// Retrieves the partition size.
    fn get_partition_size(&self) -> u64 {
        GptPartition::get_partition_size(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::Arc;

    use keramics_core::{ErrorTrace, open_os_data_stream};
    use keramics_formats::{PartitionIterator, PathComponent};

    use crate::enums::VfsFileType;
    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<GptVolumeSystem, ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    fn get_partition_file_entry(
        gpt_volume_system: &Arc<GptVolumeSystem>,
    ) -> Result<GptFileEntry, ErrorTrace> {
        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;

        Ok(GptFileEntry::Partition {
            name_index: 0,
            partition: gpt_partition,
        })
    }

    fn get_root_file_entry(gpt_volume_system: &Arc<GptVolumeSystem>) -> GptFileEntry {
        GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        }
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_none());

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let result: Option<VfsPartitionIdentifier> = file_entry.get_identifier();
        assert!(result.is_none());

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let identifier: VfsPartitionIdentifier = file_entry.get_identifier().unwrap();
        assert_eq!(
            identifier.to_string(),
            "0b119671-75ff-4e2a-a31a-0bc83f857fdd"
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("gpt1"));

        Ok(())
    }

    #[test]
    fn test_get_partition_number() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let partition_number: Option<usize> = file_entry.get_partition_number();
        assert_eq!(partition_number, None);

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let partition_number: Option<usize> = file_entry.get_partition_number();
        assert_eq!(partition_number, Some(1));
        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 1048576);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 2);

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);

        let sub_file_entry: GptFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("gpt1"));

        let result: Result<GptFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: GptFileEntry = get_root_file_entry(&gpt_volume_system);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: GptFileEntry = get_partition_file_entry(&gpt_volume_system)?;
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
