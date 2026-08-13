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

/// File entry iterator trait.
pub trait FileEntryIterator {
    /// Retrieves the number of sub file entries.
    fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace>;

    /// Retrieves a specific sub file entry.
    fn get_sub_file_entry_by_index(&mut self, index: usize) -> Result<Self, ErrorTrace>
    where
        Self: Sized;
}
