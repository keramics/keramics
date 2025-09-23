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

#[derive(Clone, Default, PartialEq)]
pub enum VfsFileType {
    BlockDevice,
    CharacterDevice,
    Device,
    Directory,
    File,
    NamedPipe,
    #[default]
    NotSet,
    Socket,
    SymbolicLink,
    Unknown,
    Whiteout,
}

#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub enum VfsType {
    Apm,
    Ext,
    #[default]
    Fake,
    Gpt,
    Mbr,
    Ntfs,
    Os,
    Qcow,
    SparseImage,
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
            VfsType::Fake => "FAKE",
            VfsType::Gpt => "GPT",
            VfsType::Mbr => "MBR",
            VfsType::Ntfs => "NTFS",
            VfsType::Os => "OS",
            VfsType::Qcow => "QCOW",
            VfsType::SparseImage => "SPARSEIMAGE",
            VfsType::Udif => "UDIF",
            VfsType::Vhd => "VHD",
            VfsType::Vhdx => "VHDX",
        }
    }
}
