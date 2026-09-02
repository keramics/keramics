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

/// Universal Disk Image Format (UDIF) segment file.
pub struct UdifSegmentFile {}

impl UdifSegmentFile {
    /// Determines the file name given a segment number.
    pub fn get_file_name(name: &String, segment_number: u32) -> String {
        if segment_number == 1 {
            format!("{}.dmg", name)
        } else {
            format!("{}.{:03}.dmgpart", name, segment_number)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_name() {
        let base_name: String = String::from("image");

        let name: String = UdifSegmentFile::get_file_name(&base_name, 1);
        assert_eq!(name, "image.dmg");

        let name: String = UdifSegmentFile::get_file_name(&base_name, 9);
        assert_eq!(name, "image.009.dmgpart");

        let name: String = UdifSegmentFile::get_file_name(&base_name, 1234);
        assert_eq!(name, "image.1234.dmgpart");
    }
}
