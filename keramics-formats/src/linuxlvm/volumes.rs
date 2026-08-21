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

use super::volume::LinuxLvmVolume;
use super::volume_system::LinuxLvmVolumeSystem;

/// Linux Logical Volume Manager (LVM) volumes iterator.
pub struct LinuxLvmVolumesIterator<'a> {
    /// Volume system.
    volume_system: &'a LinuxLvmVolumeSystem,

    /// Number of volumes.
    number_of_volumes: usize,

    /// Partititon index.
    volume_index: usize,
}

impl<'a> LinuxLvmVolumesIterator<'a> {
    /// Creates a new iterator.
    pub fn new(volume_system: &'a LinuxLvmVolumeSystem, number_of_volumes: usize) -> Self {
        Self {
            volume_system,
            number_of_volumes,
            volume_index: 0,
        }
    }
}

impl<'a> Iterator for LinuxLvmVolumesIterator<'a> {
    type Item = Result<LinuxLvmVolume, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if self.volume_index >= self.number_of_volumes {
            return None;
        }
        let item: Self::Item = self.volume_system.get_volume_by_index(self.volume_index);
        self.volume_index += 1;
        Some(item)
    }
}
