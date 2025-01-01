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

use std::io;
use std::io::Read;
use std::rc::Rc;

use keramics::formats::sparsebundle::SparseBundleImage;
use keramics::formatters::format_as_string;
use keramics::hashes::{DigestHashContext, Md5Context};
use keramics::vfs::{VfsContext, VfsFileSystem, VfsPath, VfsPathType};

fn read_media_from_image(image: &mut SparseBundleImage) -> io::Result<(u64, String)> {
    let mut data: Vec<u8> = vec![0; 35891];
    let mut md5_context: Md5Context = Md5Context::new();
    let mut media_offset: u64 = 0;

    loop {
        let read_count: usize = image.read(&mut data)?;
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
fn read_media() -> io::Result<()> {
    let mut vfs_context: VfsContext = VfsContext::new();
    let vfs_path: VfsPath = VfsPath::new(
        VfsPathType::Os,
        "./test_data/sparsebundle/hfsplus.sparsebundle/Info.plist",
        None,
    );
    let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

    let mut image = SparseBundleImage::new();
    image.open(&vfs_file_system, &vfs_path)?;

    let (media_offset, md5_hash): (u64, String) = read_media_from_image(&mut image)?;
    assert_eq!(media_offset, image.media_size);
    assert_eq!(md5_hash.as_str(), "7adf013daec71e509669a9315a6a173c");

    Ok(())
}
