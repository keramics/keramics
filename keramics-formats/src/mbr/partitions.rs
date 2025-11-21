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

use keramics_core::ErrorTrace;

use super::partition::MbrPartition;
use super::volume_system::MbrVolumeSystem;

/// Master Boot Record (MBR) partitions iterator.
pub struct MbrPartitionsIterator<'a> {
    /// Volume system.
    volume_system: &'a MbrVolumeSystem,

    /// Number of partitions.
    number_of_partitions: usize,

    /// Partititon index.
    partition_index: usize,
}

impl<'a> MbrPartitionsIterator<'a> {
    /// Creates a new iterator.
    pub fn new(volume_system: &'a MbrVolumeSystem, number_of_partitions: usize) -> Self {
        Self {
            volume_system,
            number_of_partitions,
            partition_index: 0,
        }
    }
}

impl<'a> Iterator for MbrPartitionsIterator<'a> {
    type Item = Result<MbrPartition, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if self.partition_index >= self.number_of_partitions {
            return None;
        }
        let item: Self::Item = self
            .volume_system
            .get_partition_by_index(self.partition_index);

        self.partition_index += 1;

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
