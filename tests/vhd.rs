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

use std::cell::RefCell;
use std::io;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;

use keramics::formats::vhd::VhdFile;
use keramics::formatters::format_as_string;
use keramics::hashes::{DigestHashContext, Md5Context};
use keramics::vfs::{VfsContext, VfsFileSystem, VfsPath};

fn read_media_from_file(file: &mut VhdFile) -> io::Result<(u64, String)> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    while let Ok(read_count) = file.read(&mut data) {
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
    let vfs_path: VfsPath = VfsPath::Os {
        location: "./test_data/vhd/ntfs-parent.vhd".to_string(),
    };
    let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = VhdFile::new();
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "acb42a740c63c1f72e299463375751c8");

    Ok(())
}

#[test]
fn read_media_dynamic() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();
    let vfs_path: VfsPath = VfsPath::Os {
        location: "./test_data/vhd/ntfs-dynamic.vhd".to_string(),
    };
    let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = VhdFile::new();
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "4ce30a0c21dd037023a5692d85ade033");

    Ok(())
}

#[test]
fn read_media_sparse_dynamic() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();
    let vfs_path: VfsPath = VfsPath::Os {
        location: "./test_data/vhd/ext2.vhd".to_string(),
    };
    let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = VhdFile::new();
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "3f0e360a1211f80b3f5633a68dbe98ba");

    Ok(())
}

#[test]
fn read_media_differential() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();
    let vfs_path: VfsPath = VfsPath::Os {
        location: "./test_data/vhd/ntfs-parent.vhd".to_string(),
    };
    let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut parent_file = VhdFile::new();
    parent_file.open(&vfs_file_system, &vfs_path)?;

    let mut file = VhdFile::new();
    let vfs_path: VfsPath = VfsPath::Os {
        location: "./test_data/vhd/ntfs-differential.vhd".to_string(),
    };
    file.open(&vfs_file_system, &vfs_path)?;

    file.set_parent(&Rc::new(RefCell::new(parent_file)))?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "4241cbc76e0e17517fb564238edbe415");

    Ok(())
}
