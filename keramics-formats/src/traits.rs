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

use keramics_core::ErrorTrace;

/// Block reader trait.
pub trait BlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64;

    /// Reads data from blocks.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace>;
}

/// Extended attribute iterator trait.
pub trait ExtendedAttributeIterator {
    /// Extended attribute item.
    type ExtendedAttributeItem;

    /// Retrieves the number of extended attributes.
    fn get_number_of_extended_attributes(&mut self) -> Result<usize, ErrorTrace>;

    /// Retrieves a specific extended attribute.
    fn get_extended_attribute_by_index(
        &mut self,
        extended_attribute_index: usize,
    ) -> Result<Self::ExtendedAttributeItem, ErrorTrace>;
}

/// File entry iterator trait.
pub trait FileEntryIterator {
    /// Retrieves the number of sub file entries.
    fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace>;

    /// Retrieves a specific sub file entry.
    fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<Self, ErrorTrace>
    where
        Self: Sized;
}

/// Partition iterator trait.
pub trait PartitionIterator {
    /// Partition item.
    type PartitionItem;

    /// Retrieves a specific paritition.
    fn get_partition_by_index(
        &self,
        partition_index: usize,
    ) -> Result<Self::PartitionItem, ErrorTrace>;
}
