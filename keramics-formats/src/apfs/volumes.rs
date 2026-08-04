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

use super::container::ApfsContainer;
use super::volume::ApfsVolume;

/// Apple File System (APFS) volumes iterator.
pub struct ApfsVolumesIterator<'a> {
    /// Container.
    container: &'a ApfsContainer,

    /// Number of volumes.
    number_of_volumes: usize,

    /// Volume index.
    volume_index: usize,
}

impl<'a> ApfsVolumesIterator<'a> {
    /// Creates a new iterator.
    pub fn new(container: &'a ApfsContainer, number_of_volumes: usize) -> Self {
        Self {
            container,
            number_of_volumes,
            volume_index: 0,
        }
    }
}

impl<'a> Iterator for ApfsVolumesIterator<'a> {
    type Item = Result<ApfsVolume, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if self.volume_index >= self.number_of_volumes {
            return None;
        }
        let item: Self::Item = self.container.get_volume_by_index(self.volume_index);
        self.volume_index += 1;
        Some(item)
    }
}
