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

use std::ffi::{OsStr, OsString};

use keramics_formats::fat::FatString;
use keramics_types::{ByteString, Ucs2String};

/// Virtual File System (VFS) string.
#[derive(Clone, Debug, PartialEq)]
pub enum VfsString {
    ByteString(ByteString),
    Empty,
    OsString(OsString),
    String(String),
    Ucs2String(Ucs2String),
}

impl VfsString {
    /// Determines if the VFS string is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::ByteString(byte_string) => byte_string.is_empty(),
            Self::Empty => true,
            Self::OsString(os_string) => os_string.is_empty(),
            Self::String(string) => string.is_empty(),
            Self::Ucs2String(ucs2_string) => ucs2_string.is_empty(),
        }
    }

    /// Converts the VFS string to a `String`.
    pub fn to_string(&self) -> String {
        match self {
            Self::ByteString(byte_string) => byte_string.to_string(),
            Self::Empty => String::new(),
            // TODO: change to_string_lossy to a non-lossy conversion
            Self::OsString(os_string) => os_string.to_string_lossy().to_string(),
            Self::String(string) => string.clone(),
            Self::Ucs2String(ucs2_string) => ucs2_string.to_string(),
        }
    }

    /// Converts the VFS string to an `Ucs2String`.
    pub fn to_ucs2string(&self) -> Ucs2String {
        match self {
            Self::ByteString(_) => todo!(),
            Self::Empty => Ucs2String::new(),
            Self::OsString(_) => todo!(),
            Self::String(string) => Ucs2String::from(string),
            Self::Ucs2String(ucs2_string) => ucs2_string.clone(),
        }
    }
}

impl From<&str> for VfsString {
    /// Converts a [`&str`] into a [`VfsString`]
    #[inline(always)]
    fn from(string: &str) -> Self {
        Self::String(string.to_string())
    }
}

impl From<String> for VfsString {
    /// Converts a [`String`] into a [`VfsString`]
    #[inline(always)]
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<&String> for VfsString {
    /// Converts a [`&String`] into a [`VfsString`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::String(string.clone())
    }
}

impl From<ByteString> for VfsString {
    /// Converts a [`ByteString`] into a [`VfsString`]
    #[inline(always)]
    fn from(byte_string: ByteString) -> Self {
        Self::ByteString(byte_string)
    }
}

impl From<&ByteString> for VfsString {
    /// Converts a [`&ByteString`] into a [`VfsString`]
    #[inline(always)]
    fn from(byte_string: &ByteString) -> Self {
        Self::ByteString(byte_string.clone())
    }
}

impl From<FatString> for VfsString {
    /// Converts a [`FatString`] into a [`VfsString`]
    #[inline(always)]
    fn from(fat_string: FatString) -> Self {
        match fat_string {
            FatString::ByteString(byte_string) => Self::ByteString(byte_string),
            FatString::Ucs2String(ucs2_string) => Self::Ucs2String(ucs2_string),
        }
    }
}

impl From<&FatString> for VfsString {
    /// Converts a [`&FatString`] into a [`VfsString`]
    #[inline(always)]
    fn from(fat_string: &FatString) -> Self {
        match fat_string {
            FatString::ByteString(byte_string) => Self::ByteString(byte_string.clone()),
            FatString::Ucs2String(ucs2_string) => Self::Ucs2String(ucs2_string.clone()),
        }
    }
}

impl From<OsString> for VfsString {
    /// Converts an [`OsString`] into a [`VfsString`]
    #[inline(always)]
    fn from(os_string: OsString) -> Self {
        Self::OsString(os_string)
    }
}

impl From<&OsStr> for VfsString {
    /// Converts an [`&OsStr`] into a [`VfsString`]
    #[inline(always)]
    fn from(os_str: &OsStr) -> Self {
        Self::OsString(os_str.to_os_string())
    }
}

impl From<Ucs2String> for VfsString {
    /// Converts an [`Ucs2String`] into a [`VfsString`]
    #[inline(always)]
    fn from(ucs2_string: Ucs2String) -> Self {
        Self::Ucs2String(ucs2_string)
    }
}

impl From<&Ucs2String> for VfsString {
    /// Converts an [`&Ucs2String`] into a [`VfsString`]
    #[inline(always)]
    fn from(ucs2_string: &Ucs2String) -> Self {
        Self::Ucs2String(ucs2_string.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let vfs_string: VfsString = VfsString::from("VFS string");

        assert_eq!(vfs_string, VfsString::String(String::from("VFS string")));
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("VFS string");
        let vfs_string: VfsString = VfsString::from(&test_string);

        assert_eq!(vfs_string, VfsString::String(String::from("VFS string")));
    }

    // TODO: add tests
}
