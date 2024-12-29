/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;
use std::io::Read;

use keramics::formats::vhdx::VhdxFile;
use keramics::formatters::format_as_string;
use keramics::hashes::{DigestHashContext, Md5Context};
use keramics::types::SharedValue;
use keramics::vfs::{VfsContext, VfsFileSystemReference, VfsPath, VfsPathReference, VfsPathType};

fn read_media_from_file(file: &mut VhdxFile) -> io::Result<(u64, String)> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    loop {
        let read_count: usize = file.read(&mut data)?;
        if read_count == 0 {
            break;
        }
        md5_context.update(&data[..read_count]);

        media_offset += read_count as u64;
    }
    let hash_value: Vec<u8> = md5_context.finalize();
    let hash_string: String = format_as_string(&hash_value);

    Ok((media_offset, hash_string))
}

#[test]
fn read_media_fixed() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: VfsFileSystemReference = vfs_context.open_file_system(&vfs_path)?;

    let mut file = VhdxFile::new();

    let vfs_path: VfsPathReference =
        VfsPath::new(VfsPathType::Os, "./test_data/vhdx/ntfs-parent.vhdx", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "75537374a81c40e51e6a4b812b36ce89");

    Ok(())
}

#[test]
fn read_media_dynamic() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: VfsFileSystemReference = vfs_context.open_file_system(&vfs_path)?;

    let mut file = VhdxFile::new();

    let vfs_path: VfsPathReference =
        VfsPath::new(VfsPathType::Os, "./test_data/vhdx/ntfs-dynamic.vhdx", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "20158534070142d63ee02c9ad1a9d87e");

    Ok(())
}

#[test]
fn read_media_sparse_dynamic() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: VfsFileSystemReference = vfs_context.open_file_system(&vfs_path)?;

    let mut file = VhdxFile::new();

    let vfs_path: VfsPathReference =
        VfsPath::new(VfsPathType::Os, "./test_data/vhdx/ext2.vhdx", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "196066add11fb71c4c49cf1bb50d6d24");

    Ok(())
}

#[test]
fn read_media_differential() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: VfsFileSystemReference = vfs_context.open_file_system(&vfs_path)?;

    let mut parent_file = VhdxFile::new();

    let vfs_path: VfsPathReference =
        VfsPath::new(VfsPathType::Os, "./test_data/vhdx/ntfs-parent.vhdx", None);
    parent_file.open(&vfs_file_system, &vfs_path)?;

    let mut file = VhdxFile::new();

    let vfs_path: VfsPathReference = VfsPath::new(
        VfsPathType::Os,
        "./test_data/vhdx/ntfs-differential.vhdx",
        None,
    );
    file.open(&vfs_file_system, &vfs_path)?;

    let shared_parent_file: SharedValue<VhdxFile> = SharedValue::new(parent_file);

    file.set_parent(&shared_parent_file)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "a25df0058eecd8aa1975a68eeaa0e178");

    Ok(())
}
