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

use super::backing_volume::VolsnapBackingVolume;
use super::snapshot::VolsnapSnapshot;

/// Volume Shadow Snapshot (volsnap) snapshots iterator.
pub struct VolsnapSnapshotsIterator<'a> {
    /// Volume system.
    backing_volume: &'a VolsnapBackingVolume,

    /// Number of snapshots.
    number_of_snapshots: usize,

    /// Snapshot index.
    snapshot_index: usize,
}

impl<'a> VolsnapSnapshotsIterator<'a> {
    /// Creates a new iterator.
    pub fn new(backing_volume: &'a VolsnapBackingVolume, number_of_snapshots: usize) -> Self {
        Self {
            backing_volume,
            number_of_snapshots,
            snapshot_index: 0,
        }
    }
}

impl<'a> Iterator for VolsnapSnapshotsIterator<'a> {
    type Item = Result<VolsnapSnapshot, ErrorTrace>;

    /// Retrieves the next snapshot.
    fn next(&mut self) -> Option<Self::Item> {
        if self.snapshot_index >= self.number_of_snapshots {
            return None;
        }
        let item: Self::Item = self
            .backing_volume
            .get_snapshot_by_index(self.snapshot_index);

        self.snapshot_index += 1;

        Some(item)
    }
}
