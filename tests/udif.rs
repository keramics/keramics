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
use std::rc::Rc;

use keramics::formats::udif::UdifFile;
use keramics::formatters::format_as_string;
use keramics::hashes::{DigestHashContext, Md5Context};
use keramics::vfs::{VfsContext, VfsFileSystem, VfsPath, VfsPathType};

fn read_media_from_file(file: &mut UdifFile) -> io::Result<(u64, String)> {
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
fn read_media_adc_compressed() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = UdifFile::new();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/udif/hfsplus_adc.dmg", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "08c32fd5d0fc1c2274d1c2d34185312a");

    Ok(())
}

#[test]
fn read_media_bzip2_compressed() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = UdifFile::new();

    let vfs_path: VfsPath =
        VfsPath::new(VfsPathType::Os, "./test_data/udif/hfsplus_bzip2.dmg", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "7ec785450bbc17de417be373fd5d2159");

    Ok(())
}

#[test]
fn read_media_lzfse_compressed() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = UdifFile::new();

    let vfs_path: VfsPath =
        VfsPath::new(VfsPathType::Os, "./test_data/udif/hfsplus_lzfse.dmg", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "c2c160c788676641725fd1a4b8da733b");

    Ok(())
}

#[test]
fn read_media_zlib_compressed() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut file = UdifFile::new();

    let vfs_path: VfsPath =
        VfsPath::new(VfsPathType::Os, "./test_data/udif/hfsplus_zlib.dmg", None);
    file.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_file(&mut file)?;
    assert_eq!(media_offset, file.media_size);
    assert_eq!(md5_hash.as_str(), "399bfcc39637bde7e43eb86fcc8565ae");

    Ok(())
}
