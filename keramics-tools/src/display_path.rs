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

use std::collections::HashMap;

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterDecoder;
use keramics_vfs::{
    VfsFileEntry, VfsLocation, VfsResolver, VfsResolverReference, VfsString, VfsType,
};

use crate::enums::DisplayPathType;

/// Helper for creating human readable path representations.
pub struct DisplayPath {
    /// Character translation table.
    pub translation_table: HashMap<u32, String>,

    /// Volume or partition path type
    volume_path_type: DisplayPathType,
}

impl DisplayPath {
    /// Creates a new display path helper.
    pub fn new(volume_path_type: &DisplayPathType) -> Self {
        Self {
            translation_table: Self::get_character_translation_table(),
            volume_path_type: volume_path_type.clone(),
        }
    }

    /// Escapes unprintable characters in a string.
    fn internal_escape_string(&self, string: &str) -> String {
        let mut string_parts: Vec<String> = Vec::new();

        for character_value in string.chars() {
            let safe_character: String = match self.translation_table.get(&(character_value as u32))
            {
                Some(escaped_character) => escaped_character.clone(),
                None => character_value.to_string(),
            };
            string_parts.push(safe_character);
        }
        string_parts.join("")
    }

    /// Escapes unprintable characters in a VFS string.
    pub fn escape_string(&self, vfs_string: &VfsString) -> String {
        match vfs_string {
            VfsString::Byte(byte_string) => {
                let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();

                let mut string_parts: Vec<String> = Vec::new();

                while let Some(result) = character_decoder.next() {
                    match result {
                        Ok(code_point) => {
                            let string: String = match char::from_u32(code_point as u32) {
                                Some(unicode_character) => {
                                    match self.translation_table.get(&(unicode_character as u32)) {
                                        Some(escaped_character) => escaped_character.clone(),
                                        None => unicode_character.to_string(),
                                    }
                                }
                                None => format!("\\U{{{:08x}}}", code_point),
                            };
                            string_parts.push(string);
                        }
                        Err(error) => todo!(),
                    }
                }
                string_parts.join("")
            }
            VfsString::Empty => String::new(),
            VfsString::OsString(_) => todo!(),
            VfsString::String(string) => self.internal_escape_string(string),
            VfsString::Ucs2(ucs2_string) => ucs2_string
                .elements
                .iter()
                .map(|element| match char::from_u32(*element as u32) {
                    Some(unicode_character) => {
                        match self.translation_table.get(&(unicode_character as u32)) {
                            Some(escaped_character) => escaped_character.clone(),
                            None => unicode_character.to_string(),
                        }
                    }
                    None => format!("\\U{:08x}", element),
                })
                .collect::<Vec<String>>()
                .join(""),
        }
    }

    /// Retrieves a character translation table.
    fn get_character_translation_table() -> HashMap<u32, String> {
        let mut translation_table: HashMap<u32, String> = HashMap::new();

        // Escape C0 control characters as \x##
        for character_value in 0x00..0x20 {
            let escaped_character: String = format!("\\x{:02x}", character_value);
            translation_table.insert(character_value, escaped_character);
        }
        // Escape / as \/
        translation_table.insert('/' as u32, String::from("\\/"));

        // Escape : as \:
        translation_table.insert(':' as u32, String::from("\\:"));

        // Escape \ as \\
        translation_table.insert('\\' as u32, String::from("\\\\"));

        // Escape C1 control character as \x##
        for character_value in 0x7f..0xa0 {
            let escaped_character: String = format!("\\x{:02x}", character_value);
            translation_table.insert(character_value, escaped_character);
        }
        // Escape undefined Unicode characters as \U########
        let character_values: Vec<u32> = vec![
            0xfdd0, 0xfdd1, 0xfdd2, 0xfdd3, 0xfdd4, 0xfdd5, 0xfdd6, 0xfdd7, 0xfdd8, 0xfdd9, 0xfdda,
            0xfddb, 0xfddc, 0xfddd, 0xfdde, 0xfddf, 0xfffe, 0xffff, 0x1fffe, 0x1ffff, 0x2fffe,
            0x2ffff, 0x3fffe, 0x3ffff, 0x4fffe, 0x4ffff, 0x5fffe, 0x5ffff, 0x6fffe, 0x6ffff,
            0x7fffe, 0x7ffff, 0x8fffe, 0x8ffff, 0x9fffe, 0x9ffff, 0xafffe, 0xaffff, 0xbfffe,
            0xbffff, 0xcfffe, 0xcffff, 0xdfffe, 0xdffff, 0xefffe, 0xeffff, 0xffffe, 0xfffff,
            0x10fffe, 0x10ffff,
        ];
        for character_value in character_values.iter() {
            let escaped_character: String = format!("\\U{:08x}", character_value);
            translation_table.insert(*character_value, escaped_character);
        }
        // Escape observed non-printable Unicode characters as \U########
        let character_values: Vec<u32> = vec![
            0x2028, 0x2029, 0xe000, 0xf8ff, 0xf0000, 0xffffd, 0x100000, 0x10fffd,
        ];
        for character_value in character_values.iter() {
            let escaped_character: String = format!("\\U{:08x}", character_value);
            translation_table.insert(*character_value, escaped_character);
        }
        translation_table
    }

    /// Retrieves an identifier-based display path of a VFS location.
    fn get_identifier_display_path(
        &self,
        vfs_location: &VfsLocation,
    ) -> Result<String, ErrorTrace> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();
        let result: Option<VfsFileEntry> =
            match vfs_resolver.get_file_entry_by_location(vfs_location) {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                    return Err(error);
                }
            };
        let display_path: Option<String> = match result {
            Some(VfsFileEntry::Gpt(gpt_file_entry)) => match gpt_file_entry.get_identifier() {
                Some(identifier) => Some(format!("/gpt{{{}}}", identifier.to_string())),
                _ => None,
            },
            _ => None,
        };
        match display_path {
            Some(display_path) => Ok(display_path),
            None => self.get_index_display_path(vfs_location),
        }
    }

    /// Retrieves an index-based display path of a VFS location.
    fn get_index_display_path(&self, vfs_location: &VfsLocation) -> Result<String, ErrorTrace> {
        let display_path: String = match vfs_location {
            VfsLocation::Layer {
                path,
                parent,
                vfs_type,
            } => {
                let path_string: String = path.to_string();
                match vfs_type {
                    VfsType::Apm => path_string.replace("apm", "p"),
                    VfsType::Ext | VfsType::Fat | VfsType::Ntfs => {
                        let parent_display_path: String = match self.get_path(parent) {
                            Ok(path) => path,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve parent display path"
                                );
                                return Err(error);
                            }
                        };
                        format!("{}{}", parent_display_path, path_string)
                    }
                    VfsType::Gpt => path_string.replace("gpt", "p"),
                    VfsType::Mbr => path_string.replace("mbr", "p"),
                    _ => String::new(),
                }
            }
            _ => String::new(),
        };
        Ok(display_path)
    }

    /// Retrieves a display path of a VFS location.
    pub fn get_path(&self, vfs_location: &VfsLocation) -> Result<String, ErrorTrace> {
        match &self.volume_path_type {
            DisplayPathType::Identifier => self.get_identifier_display_path(vfs_location),
            DisplayPathType::Index => self.get_index_display_path(vfs_location),
        }
        // TODO: santize path (control characters, etc.)
    }

    /// Joins the path components into a path string.
    pub fn join_path_components(&self, path_components: &Vec<VfsString>) -> String {
        if path_components.len() == 1 && path_components[0].is_empty() {
            return String::from("/");
        } else {
            path_components
                .iter()
                .map(|component| self.escape_string(component))
                .collect::<Vec<String>>()
                .join("/")
        }
    }

    /// Sets the volume path type.
    pub fn set_volume_path_type(&mut self, volume_path_type: &DisplayPathType) {
        self.volume_path_type = volume_path_type.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::Ucs2String;

    #[test]
    fn test_internal_escape_string() -> Result<(), ErrorTrace> {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        let test_string: String = String::from("test");
        let escaped_string: String = display_path.internal_escape_string(&test_string);
        assert_eq!(escaped_string, "test");

        let test_string: String = String::from("test/");
        let escaped_string: String = display_path.internal_escape_string(&test_string);
        assert_eq!(escaped_string, "test\\/");

        let test_string: String = String::from("test:");
        let escaped_string: String = display_path.internal_escape_string(&test_string);
        assert_eq!(escaped_string, "test\\:");

        let test_string: String = String::from("test\\");
        let escaped_string: String = display_path.internal_escape_string(&test_string);
        assert_eq!(escaped_string, "test\\\\");

        let test_string: String = String::from("test\u{0019}");
        let escaped_string: String = display_path.internal_escape_string(&test_string);
        assert_eq!(escaped_string, "test\\x19");

        let test_string: String = String::from("test\u{fdd0}");
        let escaped_string: String = display_path.internal_escape_string(&test_string);
        assert_eq!(escaped_string, "test\\U0000fdd0");

        Ok(())
    }

    #[test]
    fn test_escape_string() -> Result<(), ErrorTrace> {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        let test_string: VfsString = VfsString::String(String::from("test"));
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test");

        let test_string: VfsString = VfsString::Ucs2(Ucs2String {
            elements: vec![0x0074, 0x0065, 0x0073, 0x0074, 0xd800],
        });
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test\\U0000d800");

        Ok(())
    }

    // TODO: add tests
}
