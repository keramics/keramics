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

use std::collections::HashMap;
use std::fmt::Write;

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterDecoder;
use keramics_formats::{Path, PathComponent};
use keramics_vfs::{VfsFileEntry, VfsLocation, VfsResolver, VfsResolverReference, VfsType};

use crate::enums::DisplayPathType;

const C0_CONTROL_CHARACTERS: [&'static str; 32] = [
    "\\x00", "\\x01", "\\x02", "\\x03", "\\x04", "\\x05", "\\x06", "\\x07", "\\x08", "\\x09",
    "\\x0a", "\\x0b", "\\x0c", "\\x0d", "\\x0e", "\\x0f", "\\x10", "\\x11", "\\x12", "\\x13",
    "\\x14", "\\x15", "\\x16", "\\x17", "\\x18", "\\x19", "\\x1a", "\\x1b", "\\x1c", "\\x1d",
    "\\x1e", "\\x1f",
];

static C1_CONTROL_CHARACTERS: [&'static str; 33] = [
    "\\x7f", "\\x80", "\\x81", "\\x82", "\\x83", "\\x84", "\\x85", "\\x86", "\\x87", "\\x88",
    "\\x89", "\\x8a", "\\x8b", "\\x8c", "\\x8d", "\\x8e", "\\x8f", "\\x90", "\\x91", "\\x92",
    "\\x93", "\\x94", "\\x95", "\\x96", "\\x97", "\\x98", "\\x99", "\\x9a", "\\x9b", "\\x9c",
    "\\x9d", "\\x9e", "\\x9f",
];

/// Helper for creating human readable path representations.
pub struct DisplayPath {
    /// VFS resolver reference.
    vfs_resolver: VfsResolverReference,

    /// Character translation table.
    pub translation_table: HashMap<u32, &'static str>,

    /// Volume or partition path type
    volume_path_type: DisplayPathType,
}

impl DisplayPath {
    const COMPONENT_SEPARATOR: &str = "/";

    /// Creates a new display path helper.
    pub fn new(volume_path_type: &DisplayPathType) -> Self {
        Self {
            vfs_resolver: VfsResolver::current(),
            translation_table: Self::get_character_translation_table(),
            volume_path_type: volume_path_type.clone(),
        }
    }

    /// Escapes unprintable characters in a path.
    pub fn escape_path(&self, path: &Path) -> String {
        if path.is_root() {
            String::from(Self::COMPONENT_SEPARATOR)
        } else {
            path.components
                .iter()
                .map(|component| self.escape_path_component(component))
                .collect::<Vec<String>>()
                .join(Self::COMPONENT_SEPARATOR)
        }
    }

    /// Escapes unprintable characters in a path component.
    pub fn escape_path_component(&self, path_component: &PathComponent) -> String {
        match path_component {
            PathComponent::ByteString(byte_string) => {
                let mut escaped_string: String = String::with_capacity(byte_string.len() * 2);

                let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();
                while let Some(result) = character_decoder.next() {
                    match result {
                        Ok(code_points) => {
                            for code_point in code_points {
                                match char::from_u32(code_point) {
                                    Some(unicode_character) => {
                                        match self.translation_table.get(&code_point) {
                                            Some(escaped_character) => {
                                                escaped_string.push_str(escaped_character)
                                            }
                                            None => escaped_string.push(unicode_character),
                                        }
                                    }
                                    None => {
                                        _ = write!(escaped_string, "\\U{{{:08x}}}", code_point);
                                    }
                                }
                            }
                        }
                        Err(error) => return format!("{}", error),
                    }
                }
                escaped_string
            }
            PathComponent::Current => String::from("."),
            PathComponent::OsString(_) => todo!(),
            PathComponent::Parent => String::from(".."),
            PathComponent::Root => String::new(),
            PathComponent::String(string) => self.escape_string(string),
            PathComponent::Ucs2String(ucs2_string) => {
                let mut escaped_string: String =
                    String::with_capacity(ucs2_string.elements.len() * 2);

                for element in &ucs2_string.elements {
                    let code_point: u32 = *element as u32;

                    match char::from_u32(code_point) {
                        Some(unicode_character) => match self.translation_table.get(&code_point) {
                            Some(escaped_character) => escaped_string.push_str(escaped_character),
                            None => escaped_string.push(unicode_character),
                        },
                        None => {
                            _ = write!(escaped_string, "\\U{:08x}", element);
                        }
                    }
                }
                escaped_string
            }
            PathComponent::Utf16String(utf16_string) => {
                let string: String = utf16_string.to_string();

                self.escape_string(string.as_str())
            }
        }
    }

    /// Escapes unprintable characters in a string.
    fn escape_string(&self, string: &str) -> String {
        let mut escaped_string: String = String::with_capacity(string.len() * 2);

        for character_value in string.chars() {
            match self.translation_table.get(&(character_value as u32)) {
                Some(escaped_character) => escaped_string.push_str(escaped_character),
                None => escaped_string.push(character_value),
            }
        }
        escaped_string
    }

    /// Retrieves a character translation table.
    fn get_character_translation_table() -> HashMap<u32, &'static str> {
        let mut translation_table: HashMap<u32, &'static str> = HashMap::new();

        // Escape C0 control characters as \x##
        for character_value in 0x00..0x20 {
            translation_table.insert(
                character_value,
                C0_CONTROL_CHARACTERS[character_value as usize],
            );
        }
        // Escape / as \/
        translation_table.insert('/' as u32, "\\/");

        // Escape : as \:
        translation_table.insert(':' as u32, "\\:");

        // Escape \ as \\
        translation_table.insert('\\' as u32, "\\\\");

        // Escape C1 control character as \x##
        for character_value in 0x7f..0xa0 {
            translation_table.insert(
                character_value,
                C1_CONTROL_CHARACTERS[(character_value - 0x7f) as usize],
            );
        }
        // Escape undefined Unicode characters as \U########
        translation_table.insert(0x0000fdd0, "\\U0000fdd0");
        translation_table.insert(0x0000fdd1, "\\U0000fdd1");
        translation_table.insert(0x0000fdd2, "\\U0000fdd2");
        translation_table.insert(0x0000fdd3, "\\U0000fdd3");
        translation_table.insert(0x0000fdd4, "\\U0000fdd4");
        translation_table.insert(0x0000fdd5, "\\U0000fdd5");
        translation_table.insert(0x0000fdd6, "\\U0000fdd6");
        translation_table.insert(0x0000fdd7, "\\U0000fdd7");
        translation_table.insert(0x0000fdd8, "\\U0000fdd8");
        translation_table.insert(0x0000fdd9, "\\U0000fdd9");
        translation_table.insert(0x0000fdda, "\\U0000fdda");
        translation_table.insert(0x0000fddb, "\\U0000fddb");
        translation_table.insert(0x0000fddc, "\\U0000fddc");
        translation_table.insert(0x0000fddd, "\\U0000fddd");
        translation_table.insert(0x0000fdde, "\\U0000fdde");
        translation_table.insert(0x0000fddf, "\\U0000fddf");
        translation_table.insert(0x0000fffe, "\\U0000fffe");
        translation_table.insert(0x0000ffff, "\\U0000ffff");
        translation_table.insert(0x0001fffe, "\\U0001fffe");
        translation_table.insert(0x0001ffff, "\\U0001ffff");
        translation_table.insert(0x0002fffe, "\\U0002fffe");
        translation_table.insert(0x0002ffff, "\\U0002ffff");
        translation_table.insert(0x0003fffe, "\\U0003fffe");
        translation_table.insert(0x0003ffff, "\\U0003ffff");
        translation_table.insert(0x0004fffe, "\\U0004fffe");
        translation_table.insert(0x0004ffff, "\\U0004ffff");
        translation_table.insert(0x0005fffe, "\\U0005fffe");
        translation_table.insert(0x0005ffff, "\\U0005ffff");
        translation_table.insert(0x0006fffe, "\\U0006fffe");
        translation_table.insert(0x0006ffff, "\\U0006ffff");
        translation_table.insert(0x0007fffe, "\\U0007fffe");
        translation_table.insert(0x0007ffff, "\\U0007ffff");
        translation_table.insert(0x0008fffe, "\\U0008fffe");
        translation_table.insert(0x0008ffff, "\\U0008ffff");
        translation_table.insert(0x0009fffe, "\\U0009fffe");
        translation_table.insert(0x0009ffff, "\\U0009ffff");
        translation_table.insert(0x000afffe, "\\U000afffe");
        translation_table.insert(0x000affff, "\\U000affff");
        translation_table.insert(0x000bfffe, "\\U000bfffe");
        translation_table.insert(0x000bffff, "\\U000bffff");
        translation_table.insert(0x000cfffe, "\\U000cfffe");
        translation_table.insert(0x000cffff, "\\U000cffff");
        translation_table.insert(0x000dfffe, "\\U000dfffe");
        translation_table.insert(0x000dffff, "\\U000dffff");
        translation_table.insert(0x000efffe, "\\U000efffe");
        translation_table.insert(0x000effff, "\\U000effff");
        translation_table.insert(0x000ffffe, "\\U000ffffe");
        translation_table.insert(0x000fffff, "\\U000fffff");
        translation_table.insert(0x0010fffe, "\\U0010fffe");
        translation_table.insert(0x0010ffff, "\\U0010ffff");

        // Escape observed non-printable Unicode characters as \U########
        translation_table.insert(0x00002028, "\\U00002028");
        translation_table.insert(0x00002029, "\\U00002029");
        translation_table.insert(0x0000e000, "\\U0000e000");
        translation_table.insert(0x0000f8ff, "\\U0000f8ff");
        translation_table.insert(0x000f0000, "\\U000f0000");
        translation_table.insert(0x000ffffd, "\\U000ffffd");
        translation_table.insert(0x00100000, "\\U00100000");
        translation_table.insert(0x0010fffd, "\\U0010fffd");

        translation_table
    }

    /// Retrieves an identifier-based display path of a VFS location.
    fn get_identifier_display_path(
        &self,
        vfs_location: &VfsLocation,
    ) -> Result<String, ErrorTrace> {
        let display_path: Option<String> = match vfs_location {
            VfsLocation::Layer {
                parent, vfs_type, ..
            } => match vfs_type {
                VfsType::ApfsContainer | VfsType::Gpt | VfsType::LinuxLvm => {
                    match self.vfs_resolver.get_file_entry_by_location(vfs_location) {
                        Ok(vfs_file_entry) => match vfs_file_entry {
                            Some(VfsFileEntry::ApfsContainer(apfs_container_file_entry)) => {
                                match apfs_container_file_entry.get_identifier() {
                                    Some(identifier) => {
                                        let path_string: String =
                                            format!("/apfs{{{}}}", identifier);

                                        match self.get_path(parent) {
                                            Ok(parent_display_path) => Some(format!(
                                                "{}{}",
                                                parent_display_path, path_string
                                            )),
                                            Err(mut error) => {
                                                keramics_core::error_trace_add_frame!(
                                                    error,
                                                    "Unable to retrieve parent display path"
                                                );
                                                return Err(error);
                                            }
                                        }
                                    }
                                    None => None,
                                }
                            }
                            Some(VfsFileEntry::Gpt(gpt_file_entry)) => {
                                match gpt_file_entry.get_identifier() {
                                    Some(identifier) => Some(format!("/gpt{{{}}}", identifier)),
                                    None => None,
                                }
                            }
                            Some(VfsFileEntry::LinuxLvm(lvm_file_entry)) => {
                                match lvm_file_entry.get_identifier() {
                                    Some(identifier) => Some(format!("/lvm{{{}}}", identifier)),
                                    None => None,
                                }
                            }
                            _ => None,
                        },
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve file entry"
                            );
                            return Err(error);
                        }
                    }
                }
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
        let display_path: Option<String> = match vfs_location {
            VfsLocation::Layer {
                path,
                parent,
                vfs_type,
            } => {
                let path_string: String = path.to_string();

                match vfs_type {
                    VfsType::Apfs
                    | VfsType::ApfsContainer
                    | VfsType::Ext
                    | VfsType::Fat
                    | VfsType::Hfs
                    | VfsType::LinuxLvm
                    | VfsType::Ntfs => match self.get_path(parent) {
                        Ok(parent_display_path) => {
                            Some(format!("{}{}", parent_display_path, path_string))
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve parent display path"
                            );
                            return Err(error);
                        }
                    },
                    VfsType::Apm => Some(path_string.replace("apm", "p")),
                    VfsType::Gpt | VfsType::Mbr => {
                        match self.vfs_resolver.get_file_entry_by_location(vfs_location) {
                            Ok(vfs_file_entry) => match vfs_file_entry {
                                Some(VfsFileEntry::Gpt(gpt_file_entry)) => {
                                    match gpt_file_entry.get_partition_number() {
                                        Some(partition_number) => {
                                            Some(format!("/p{}", partition_number))
                                        }
                                        None => Some(path_string.replace("gpt", "p")),
                                    }
                                }
                                Some(VfsFileEntry::Mbr(mbr_file_entry)) => {
                                    match mbr_file_entry.get_partition_number() {
                                        Some(partition_number) => {
                                            Some(format!("/p{}", partition_number))
                                        }
                                        None => Some(path_string.replace("mbr", "p")),
                                    }
                                }
                                _ => None,
                            },
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve file entry"
                                );
                                return Err(error);
                            }
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        match display_path {
            Some(display_path) => Ok(display_path),
            None => Ok(String::new()),
        }
    }

    /// Retrieves a display path of a VFS location.
    pub fn get_path(&self, vfs_location: &VfsLocation) -> Result<String, ErrorTrace> {
        match &self.volume_path_type {
            DisplayPathType::Identifier => self.get_identifier_display_path(vfs_location),
            DisplayPathType::Index => self.get_index_display_path(vfs_location),
        }
        // TODO: santize path (control characters, etc.)
    }

    /// Sets the volume path type.
    #[allow(dead_code)]
    pub fn set_volume_path_type(&mut self, volume_path_type: &DisplayPathType) {
        self.volume_path_type = volume_path_type.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_formats::Path;
    use keramics_types::{ByteString, Ucs2String};

    #[test]
    fn test_escape_path() {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        let path: Path = Path::from("/");
        assert!(path.is_root());
        let escaped_path: String = display_path.escape_path(&path);
        assert_eq!(escaped_path, "/");

        let path: Path = Path::from("/foo/bar");
        let escaped_path: String = display_path.escape_path(&path);
        assert_eq!(escaped_path, "/foo/bar");

        let path: Path = Path::from("foo/bar");
        let escaped_path: String = display_path.escape_path(&path);
        assert_eq!(escaped_path, "foo/bar");

        let escaped_path: Path = Path {
            components: vec![
                PathComponent::Root,
                PathComponent::String(String::from("foo/bar")),
                PathComponent::Parent,
                PathComponent::Current,
                PathComponent::ByteString(ByteString::from("byte:string")),
            ],
        };
        let escaped_path: String = display_path.escape_path(&escaped_path);
        assert_eq!(escaped_path, "/foo\\/bar/.././byte\\:string");
    }

    #[test]
    fn test_escape_path_component() {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        let test_path_component: PathComponent = PathComponent::from("test");
        let escaped_string: String = display_path.escape_path_component(&test_path_component);
        assert_eq!(escaped_string, "test");

        let test_path_component: PathComponent = PathComponent::from(Ucs2String {
            elements: vec![0x0074, 0x0065, 0x0073, 0x0074, 0xd800],
        });
        let escaped_string: String = display_path.escape_path_component(&test_path_component);
        assert_eq!(escaped_string, "test\\U0000d800");
    }

    #[test]
    fn test_escape_string() {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        let test_string: String = String::from("test");
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test");

        let test_string: String = String::from("test/");
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test\\/");

        let test_string: String = String::from("test:");
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test\\:");

        let test_string: String = String::from("test\\");
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test\\\\");

        let test_string: String = String::from("test\u{0019}");
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test\\x19");

        let test_string: String = String::from("test\u{fdd0}");
        let escaped_string: String = display_path.escape_string(&test_string);
        assert_eq!(escaped_string, "test\\U0000fdd0");
    }

    #[test]
    fn test_get_character_translation_table() {
        let _ = DisplayPath::get_character_translation_table();
    }

    #[test]
    fn test_get_identifier_display_path() -> Result<(), ErrorTrace> {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Identifier);

        let os_vfs_location: VfsLocation = VfsLocation::from("../test_data/gpt/gpt.raw");

        let test_path: String = display_path.get_identifier_display_path(&os_vfs_location)?;
        assert_eq!(test_path, String::from(""));

        let path: Path = Path::from("/gpt1");
        let gpt_vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Gpt, path);

        let test_path: String = display_path.get_identifier_display_path(&gpt_vfs_location)?;
        assert_eq!(
            test_path,
            String::from("/gpt{0b119671-75ff-4e2a-a31a-0bc83f857fdd}")
        );

        let os_vfs_location: VfsLocation = VfsLocation::from("../test_data/mbr/mbr.raw");

        let test_path: String = display_path.get_identifier_display_path(&os_vfs_location)?;
        assert_eq!(test_path, String::from(""));

        let path: Path = Path::from("/mbr1");
        let mbr_vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Mbr, path);

        let test_path: String = display_path.get_identifier_display_path(&mbr_vfs_location)?;
        assert_eq!(test_path, String::from("/p1"));

        Ok(())
    }

    #[test]
    fn test_get_index_display_path() -> Result<(), ErrorTrace> {
        let display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        let os_vfs_location: VfsLocation = VfsLocation::from("../test_data/gpt/gpt.raw");

        let test_path: String = display_path.get_index_display_path(&os_vfs_location)?;
        assert_eq!(test_path, String::from(""));

        let path: Path = Path::from("/gpt1");
        let gpt_vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Gpt, path);

        let test_path: String = display_path.get_index_display_path(&gpt_vfs_location)?;
        assert_eq!(test_path, String::from("/p1"));

        let os_vfs_location: VfsLocation = VfsLocation::from("../test_data/mbr/mbr.raw");

        let test_path: String = display_path.get_index_display_path(&os_vfs_location)?;
        assert_eq!(test_path, String::from(""));

        let path: Path = Path::from("/mbr1");
        let mbr_vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Mbr, path);

        let test_path: String = display_path.get_index_display_path(&mbr_vfs_location)?;
        assert_eq!(test_path, String::from("/p1"));

        Ok(())
    }

    #[test]
    fn test_get_path() -> Result<(), ErrorTrace> {
        let mut display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Identifier);

        let os_vfs_location: VfsLocation = VfsLocation::from("../test_data/gpt/gpt.raw");
        let path: Path = Path::from("/gpt1");
        let gpt_vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Gpt, path);

        let test_path: String = display_path.get_path(&gpt_vfs_location)?;
        assert_eq!(
            test_path,
            String::from("/gpt{0b119671-75ff-4e2a-a31a-0bc83f857fdd}")
        );

        display_path.set_volume_path_type(&DisplayPathType::Index);

        let test_path: String = display_path.get_path(&gpt_vfs_location)?;
        assert_eq!(test_path, String::from("/p1"));

        Ok(())
    }

    #[test]
    fn test_set_volume_path_type() {
        let mut display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        assert_eq!(display_path.volume_path_type, DisplayPathType::Index);

        display_path.set_volume_path_type(&DisplayPathType::Identifier);
        assert_eq!(display_path.volume_path_type, DisplayPathType::Identifier);
    }
}
