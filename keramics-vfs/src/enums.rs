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

impl VfsType {
    /// Retrieves a string representation of the type.
    pub fn as_str(&self) -> &str {
        match self {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_type_as_str() {
        let vfs_type: VfsType = VfsType::Apm;
        assert_eq!(vfs_type.as_str(), "APM");

        let vfs_type: VfsType = VfsType::Ext;
        assert_eq!(vfs_type.as_str(), "EXT");

        let vfs_type: VfsType = VfsType::Ewf;
        assert_eq!(vfs_type.as_str(), "EWF");

        let vfs_type: VfsType = VfsType::Fake;
        assert_eq!(vfs_type.as_str(), "FAKE");

        let vfs_type: VfsType = VfsType::Fat;
        assert_eq!(vfs_type.as_str(), "FAT");

        let vfs_type: VfsType = VfsType::Gpt;
        assert_eq!(vfs_type.as_str(), "GPT");

        let vfs_type: VfsType = VfsType::Mbr;
        assert_eq!(vfs_type.as_str(), "MBR");

        let vfs_type: VfsType = VfsType::Ntfs;
        assert_eq!(vfs_type.as_str(), "NTFS");

        let vfs_type: VfsType = VfsType::Os;
        assert_eq!(vfs_type.as_str(), "OS");

        let vfs_type: VfsType = VfsType::Qcow;
        assert_eq!(vfs_type.as_str(), "QCOW");

        let vfs_type: VfsType = VfsType::SparseImage;
        assert_eq!(vfs_type.as_str(), "SPARSEIMAGE");

        let vfs_type: VfsType = VfsType::SplitRaw;
        assert_eq!(vfs_type.as_str(), "SPLITRAW");

        let vfs_type: VfsType = VfsType::Udif;
        assert_eq!(vfs_type.as_str(), "UDIF");

        let vfs_type: VfsType = VfsType::Vhd;
        assert_eq!(vfs_type.as_str(), "VHD");

        let vfs_type: VfsType = VfsType::Vhdx;
        assert_eq!(vfs_type.as_str(), "VHDX");
    }
}
