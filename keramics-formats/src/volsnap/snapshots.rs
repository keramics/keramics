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

use super::shadow_storage::VolsnapShadowStorage;
use super::snapshot::VolsnapSnapshot;

/// Volume Shadow Snapshot (volsnap) snapshots iterator.
pub struct VolsnapSnapshotsIterator<'a> {
    /// Shadow storage.
    shadow_storage: &'a VolsnapShadowStorage,

    /// Number of snapshots.
    number_of_snapshots: usize,

    /// Snapshot index.
    snapshot_index: usize,
}

impl<'a> VolsnapSnapshotsIterator<'a> {
    /// Creates a new iterator.
    pub fn new(shadow_storage: &'a VolsnapShadowStorage, number_of_snapshots: usize) -> Self {
        Self {
            shadow_storage,
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
            .shadow_storage
            .get_snapshot_by_index(self.snapshot_index);

        self.snapshot_index += 1;

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        let shadow_storage: VolsnapShadowStorage = VolsnapShadowStorage::new();

        // Test with zero snapshots.
        let mut iterator: VolsnapSnapshotsIterator =
            VolsnapSnapshotsIterator::new(&shadow_storage, 0);
        assert!(iterator.next().is_none());

        // Test iterating past available snapshots returns error items then None.
        let mut iterator: VolsnapSnapshotsIterator =
            VolsnapSnapshotsIterator::new(&shadow_storage, 1);
        let first_item: Option<Result<VolsnapSnapshot, ErrorTrace>> = iterator.next();
        assert!(first_item.is_some());
        let result: Result<VolsnapSnapshot, ErrorTrace> = first_item.unwrap();
        assert!(result.is_err());
        assert!(iterator.next().is_none());
    }
}
