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

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum VfsFileType {
    BlockDevice,
    CharacterDevice,
    Device,
    Directory,
    File,
    NamedPipe,
    Socket,
    SymbolicLink,
    Unknown(u16),
    Whiteout,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VfsType {
    Apm,
    Ext,
    Ewf,
    Fake,
    Fat,
    Gpt,
    Mbr,
    Ntfs,
    Os,
    Qcow,
    SparseImage,
    SplitRaw,
    Udif,
    Vhd,
    Vhdx,
}

impl fmt::Display for VfsType {
    /// Formats the VFS type for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: &str = match self {
            VfsType::Apm => "APM",
            VfsType::Ext => "EXT",
            VfsType::Ewf => "EWF",
            VfsType::Fake => "FAKE",
            VfsType::Fat => "FAT",
            VfsType::Gpt => "GPT",
            VfsType::Mbr => "MBR",
            VfsType::Ntfs => "NTFS",
            VfsType::Os => "OS",
            VfsType::Qcow => "QCOW",
            VfsType::SparseImage => "SPARSEIMAGE",
            VfsType::SplitRaw => "SPLITRAW",
            VfsType::Udif => "UDIF",
            VfsType::Vhd => "VHD",
            VfsType::Vhdx => "VHDX",
        };
        write!(formatter, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_type_fmt() {
        let vfs_type: VfsType = VfsType::Apm;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "APM");

        let vfs_type: VfsType = VfsType::Ext;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "EXT");

        let vfs_type: VfsType = VfsType::Ewf;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "EWF");

        let vfs_type: VfsType = VfsType::Fake;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "FAKE");

        let vfs_type: VfsType = VfsType::Fat;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "FAT");

        let vfs_type: VfsType = VfsType::Gpt;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "GPT");

        let vfs_type: VfsType = VfsType::Mbr;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "MBR");

        let vfs_type: VfsType = VfsType::Ntfs;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "NTFS");

        let vfs_type: VfsType = VfsType::Os;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "OS");

        let vfs_type: VfsType = VfsType::Qcow;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "QCOW");

        let vfs_type: VfsType = VfsType::SparseImage;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "SPARSEIMAGE");

        let vfs_type: VfsType = VfsType::SplitRaw;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "SPLITRAW");

        let vfs_type: VfsType = VfsType::Udif;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "UDIF");

        let vfs_type: VfsType = VfsType::Vhd;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "VHD");

        let vfs_type: VfsType = VfsType::Vhdx;
        let string: String = vfs_type.to_string();
        assert_eq!(string, "VHDX");
    }
}
