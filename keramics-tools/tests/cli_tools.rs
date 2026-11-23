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

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Child, Command, Output, Stdio};

/// Executes a tool binary with the given command line arguments.
fn run_binary(binary_path: &str, arguments: &[&str]) -> Output {
    Command::new(binary_path)
        .args(arguments)
        .output()
        .expect("Failed to execute binary")
}

/// Executes `keramicsinfo` on each case and verifies exit status and expected output header.
fn assert_keramicsinfo_cases(cases: &[(&[&str], &str)]) {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsinfo");

    for (arguments, expected_header) in cases {
        let output: Output = run_binary(binary_path, arguments);
        let standard_output: String = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "keramicsinfo failed on {:?}: stdout={} stderr={}",
            arguments,
            standard_output,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            standard_output.contains(expected_header),
            "keramicsinfo output for {:?} did not contain {:?}: {}",
            arguments,
            expected_header,
            standard_output
        );
    }
}

#[test]
fn test_keramicsinfo_file_systems() {
    let cases: [(&[&str], &str); 14] = [
        (
            &["--format", "apfs", "../test_data/apfs/apfs.raw"],
            "Apple File System (APFS) information:",
        ),
        (
            &["--format", "exfat", "../test_data/exfat/exfat.raw"],
            "Extensible File Allocation Table (exFAT) information:",
        ),
        (
            &[
                "--format",
                "ext",
                "--encoding",
                "utf8",
                "../test_data/ext/ext2.raw",
            ],
            "Extended File System (ext) information:",
        ),
        (
            &["--format", "ext", "../test_data/ext/ext3.raw"],
            "Extended File System (ext) information:",
        ),
        (
            &["--format", "ext", "../test_data/ext/ext4.raw"],
            "Extended File System (ext) information:",
        ),
        (
            &["--format", "fat", "../test_data/fat/fat12.raw"],
            "File Allocation Table (FAT) information:",
        ),
        (
            &[
                "--contents",
                "--format",
                "fat",
                "../test_data/qcow/fat16.qcow2",
            ],
            "File Allocation Table (FAT) information:",
        ),
        (
            &[
                "--contents",
                "--format",
                "fat",
                "../test_data/qcow/fat32.qcow2",
            ],
            "File Allocation Table (FAT) information:",
        ),
        (
            &["--format", "hfs", "../test_data/hfs/hfs.raw"],
            "Hierarchical File System (HFS) information:",
        ),
        (
            &["--format", "hfs", "../test_data/hfs/hfsplus.raw"],
            "Hierarchical File System (HFS) information:",
        ),
        (
            &[
                "--format",
                "hfs",
                "../test_data/hfs/hfs_wrapped_hfsplus.raw",
            ],
            "Hierarchical File System (HFS) information:",
        ),
        (
            &["--format", "ntfs", "../test_data/ntfs/ntfs.raw"],
            "New Technologies File System (NTFS) information:",
        ),
        (
            &["--format", "xfs", "../test_data/xfs/xfs.raw"],
            "X File System (XFS) information:",
        ),
        (
            &["--contents", "--format", "ext", "../test_data/vhd/ext2.vhd"],
            "Extended File System (ext) information:",
        ),
    ];
    assert_keramicsinfo_cases(&cases);
}

#[test]
fn test_keramicsinfo_volume_systems_and_archives() {
    let cases: [(&[&str], &str); 14] = [
        (
            &["--format", "apm", "../test_data/apm/apm.dmg"],
            "Apple Partition Map (APM) information:",
        ),
        (
            &[
                "--contents",
                "--offset",
                "0x10000",
                "--format",
                "bde",
                "../test_data/bde/bde_aes128.vhd",
            ],
            "BitLocker Drive Encryption (BDE) information:",
        ),
        (
            &[
                "--contents",
                "--offset",
                "0x10000",
                "--format",
                "bde",
                "../test_data/bde/bde_aes128_used_space.vhd",
            ],
            "BitLocker Drive Encryption (BDE) information:",
        ),
        (
            &["--format", "bsdlabel", "../test_data/bsdlabel/bsdlabel.raw"],
            "BSD disklabel (bsdlabel) information:",
        ),
        (
            &[
                "--format",
                "cdsaencr",
                "../test_data/udif/hfsplus_aes256.dmg",
            ],
            "Encrypted Encoding container (cdsaencr) information:",
        ),
        (
            &["--format", "cpio", "../test_data/cpio/newc.cpio"],
            "Copy in and out (CPIO) archive information:",
        ),
        (
            &["--format", "cpio", "../test_data/cpio/bin_le.cpio"],
            "Copy in and out (CPIO) archive information:",
        ),
        (
            &["--format", "cpio", "../test_data/cpio/odc.cpio"],
            "Copy in and out (CPIO) archive information:",
        ),
        (
            &["--format", "gpt", "../test_data/gpt/gpt.raw"],
            "GUID Partition Table (GPT) information:",
        ),
        (
            &["--format", "linuxlvm", "../test_data/linuxlvm/lvm2.raw"],
            "Linux Logical Volume Manager (LVM) information:",
        ),
        (
            &["--format", "luksde", "../test_data/luksde/luks1.raw"],
            "Linux Unified Key Setup (LUKS) Disk Encryption information:",
        ),
        (
            &["--format", "luksde", "../test_data/luksde/luks2.raw"],
            "Linux Unified Key Setup (LUKS) Disk Encryption information:",
        ),
        (
            &["--format", "mbr", "../test_data/mbr/mbr.raw"],
            "Master Boot Record (MBR) information:",
        ),
        (
            &["--format", "sgilabel", "../test_data/sgilabel/sgilabel.raw"],
            "SGI disklabel (sgilabel) information:",
        ),
    ];
    assert_keramicsinfo_cases(&cases);

    let volsnap_cases: [(&[&str], &str); 1] = [(
        &[
            "--contents",
            "--offset",
            "0x10000",
            "--format",
            "volsnap",
            "../test_data/volsnap/volsnap.vhd",
        ],
        "Volume Shadow Snapshot (volsnap) information:",
    )];
    assert_keramicsinfo_cases(&volsnap_cases);
}

#[test]
fn test_keramicsinfo_storage_media_images() {
    let cases: [(&[&str], &str); 21] = [
        (
            &["--format", "ewf", "../test_data/ewf/ext2.E01"],
            "Expert Witness Compression Format (EWF) information:",
        ),
        (
            &["--format", "ewf", "../test_data/ewf/ntfs.L01"],
            "Expert Witness Compression Format (EWF) information:",
        ),
        (
            &[
                "--format",
                "pdi",
                "../test_data/pdi/hfsplus.hdd/DiskDescriptor.xml",
            ],
            "Parallels Disk Image (PDI) information:",
        ),
        (
            &["--format", "qcow", "../test_data/qcow/ext2.qcow"],
            "QEMU Copy-On-Write (QCOW) information:",
        ),
        (
            &["--format", "qcow", "../test_data/qcow/ext2.qcow2"],
            "QEMU Copy-On-Write (QCOW) information:",
        ),
        (
            &["--format", "qcow", "../test_data/qcow/ext2_zlib.qcow2"],
            "QEMU Copy-On-Write (QCOW) information:",
        ),
        (
            &["--format", "qcow", "../test_data/qcow/ext2_aes128.qcow2"],
            "QEMU Copy-On-Write (QCOW) information:",
        ),
        (
            &[
                "--format",
                "sparsebundle",
                "../test_data/sparsebundle/hfsplus.sparsebundle",
            ],
            "Sparse bundle (.sparsebundle) information:",
        ),
        (
            &[
                "--format",
                "sparsebundle",
                "../test_data/sparsebundle/hfsplus_aes128.sparsebundle",
            ],
            "Sparse bundle (.sparsebundle) information:",
        ),
        (
            &[
                "--format",
                "sparseimage",
                "../test_data/sparseimage/hfsplus.sparseimage",
            ],
            "Sparse image (.sparseimage) information:",
        ),
        (
            &["--format", "udif", "../test_data/udif/hfsplus_zlib.dmg"],
            "Universal Disk Image Format (UDIF) information:",
        ),
        (
            &["--format", "udif", "../test_data/udif/hfsplus_adc.dmg"],
            "Universal Disk Image Format (UDIF) information:",
        ),
        (
            &["--format", "udif", "../test_data/udif/hfsplus_bzip2.dmg"],
            "Universal Disk Image Format (UDIF) information:",
        ),
        (
            &["--format", "udif", "../test_data/udif/hfsplus_lzfse.dmg"],
            "Universal Disk Image Format (UDIF) information:",
        ),
        (
            &["--format", "udif", "../test_data/udif/hfsplus_segments.dmg"],
            "Universal Disk Image Format (UDIF) information:",
        ),
        (
            &["--format", "vdi", "../test_data/vdi/ext2.vdi"],
            "Virtual Disk Image (VDI) information:",
        ),
        (
            &["--format", "vhd", "../test_data/vhd/ext2.vhd"],
            "Virtual Hard Disk (VHD) information:",
        ),
        (
            &["--format", "vhd", "../test_data/vhd/ntfs-differential.vhd"],
            "Virtual Hard Disk (VHD) information:",
        ),
        (
            &["--format", "vhdx", "../test_data/vhdx/ext2.vhdx"],
            "Virtual Hard Disk (VHDX) information:",
        ),
        (
            &[
                "--format",
                "vhdx",
                "../test_data/vhdx/ntfs-differential.vhdx",
            ],
            "Virtual Hard Disk (VHDX) information:",
        ),
        (
            &["--format", "vmdk", "../test_data/vmdk/ext2.vmdk"],
            "VMware Virtual Disk (VMDK) information:",
        ),
    ];
    assert_keramicsinfo_cases(&cases);
}

#[test]
fn test_keramicsinfo_auto_detect() {
    let cases: [(&[&str], &str); 10] = [
        // Test header signature auto-detection.
        (
            &["../test_data/ext/ext2.raw"],
            "Extended File System (ext) information:",
        ),
        // Test footer signature auto-detection.
        (
            &["../test_data/udif/hfsplus_zlib.dmg"],
            "Universal Disk Image Format (UDIF) information:",
        ),
        (
            &["../test_data/vhd/ext2.vhd"],
            "Virtual Hard Disk (VHD) information:",
        ),
        // Test MBR fallback signature auto-detection.
        (
            &["../test_data/mbr/mbr.raw"],
            "Master Boot Record (MBR) information:",
        ),
        // Test locked Mac OS Encrypted Encoding container auto-detection.
        (
            &["../test_data/udif/hfsplus_aes256.dmg"],
            "Encrypted Encoding container (cdsaencr) information:",
        ),
        (
            &["../test_data/sparseimage/hfsplus_aes128.sparseimage"],
            "Encrypted Encoding container (cdsaencr) information:",
        ),
        // Test sparse bundle directory auto-detection.
        (
            &["../test_data/sparsebundle/hfsplus.sparsebundle"],
            "Sparse bundle (.sparsebundle) information:",
        ),
        (
            &["../test_data/sparsebundle/hfsplus_aes128.sparsebundle"],
            "Sparse bundle (.sparsebundle) information:",
        ),
        // Test contents mode auto-detection.
        (
            &["--contents", "../test_data/qcow/fat16.qcow2"],
            "File Allocation Table (FAT) information:",
        ),
        (
            &["--contents", "../test_data/vhd/ext2.vhd"],
            "Extended File System (ext) information:",
        ),
    ];
    assert_keramicsinfo_cases(&cases);
}

#[test]
fn test_keramicsinfo_encrypted_qcow_and_apfs() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsinfo");

    for arguments in [
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "../test_data/qcow/ext2_aes128.qcow2",
        ],
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "../test_data/apfs/apfs_encrypted.raw",
        ],
    ] {
        let output: Output = run_binary(binary_path, &arguments);
        assert!(
            output.status.success(),
            "keramicsinfo failed on {:?}: stdout={} stderr={}",
            arguments,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_keramicsinfo_encrypted_udif_and_sparse() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsinfo");

    for arguments in [
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "../test_data/udif/hfsplus_aes256.dmg",
        ],
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "../test_data/sparsebundle/hfsplus_aes128.sparsebundle",
        ],
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "../test_data/sparseimage/hfsplus_aes128.sparseimage",
        ],
    ] {
        let output: Output = run_binary(binary_path, &arguments);
        assert!(
            output.status.success(),
            "keramicsinfo failed on {:?}: stdout={} stderr={}",
            arguments,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_keramicsinfo_subcommands_and_errors() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsinfo");

    let success_cases: Vec<Vec<&str>> = vec![
        // Test hierarchy subcommand.
        vec!["../test_data/apfs/apfs.raw", "hierarchy"],
        vec![
            "../test_data/apfs/apfs.raw",
            "hierarchy",
            "--volume",
            "1",
            "--volume-path-type",
            "identifier",
        ],
        vec!["../test_data/exfat/exfat.raw", "hierarchy"],
        vec![
            "../test_data/exfat/exfat.raw",
            "hierarchy",
            "--path",
            "/testdir1",
        ],
        vec!["../test_data/ext/ext2.raw", "hierarchy"],
        vec![
            "../test_data/ext/ext2.raw",
            "hierarchy",
            "--path",
            "/testdir1",
        ],
        vec!["../test_data/fat/fat12.raw", "hierarchy"],
        vec![
            "../test_data/fat/fat12.raw",
            "hierarchy",
            "--path",
            "/testdir1",
        ],
        vec!["../test_data/hfs/hfsplus.raw", "hierarchy"],
        vec![
            "../test_data/hfs/hfsplus.raw",
            "hierarchy",
            "--path",
            "/testdir1",
        ],
        vec!["../test_data/ntfs/ntfs.raw", "hierarchy"],
        vec![
            "../test_data/ntfs/ntfs.raw",
            "hierarchy",
            "--path",
            "/testdir1",
        ],
        vec!["../test_data/xfs/xfs.raw", "hierarchy"],
        vec![
            "../test_data/xfs/xfs.raw",
            "hierarchy",
            "--path",
            "/testdir1",
        ],
        // Test path subcommand.
        vec!["../test_data/apfs/apfs.raw", "path", "/testdir1/testfile1"],
        vec![
            "../test_data/exfat/exfat.raw",
            "path",
            "/testdir1/testfile1",
        ],
        vec!["../test_data/ext/ext2.raw", "path", "/testdir1/testfile1"],
        vec!["../test_data/fat/fat12.raw", "path", "/testdir1/testfile1"],
        vec![
            "../test_data/hfs/hfsplus.raw",
            "path",
            "/testdir1/testfile1",
        ],
        vec!["../test_data/ntfs/ntfs.raw", "path", "/testdir1/testfile1"],
        vec!["../test_data/xfs/xfs.raw", "path", "/testdir1/testfile1"],
        // Test identifier subcommand.
        vec!["../test_data/apfs/apfs.raw", "identifier", "18"],
        vec!["../test_data/exfat/exfat.raw", "identifier", "0x00201c00"],
        vec!["../test_data/ext/ext2.raw", "identifier", "14"],
        vec!["../test_data/fat/fat12.raw", "identifier", "0x00006260"],
        vec!["../test_data/hfs/hfsplus.raw", "identifier", "20"],
        vec!["../test_data/ntfs/ntfs.raw", "identifier", "66"],
        vec!["../test_data/xfs/xfs.raw", "identifier", "16133"],
    ];
    for arguments in &success_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(
            output.status.success(),
            "keramicsinfo failed on {:?}: stdout={} stderr={}",
            arguments,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test error cases.
    let error_cases: Vec<Vec<&str>> = vec![
        vec!["../test_data/nonexistent.raw"],
        vec!["../test_data/directory/file.txt"],
        vec!["--credential", "missing_colon", "../test_data/ext/ext2.raw"],
        vec![
            "--credential",
            "unknown:secret",
            "../test_data/ext/ext2.raw",
        ],
        vec!["../test_data/mbr/mbr.raw", "hierarchy"],
        vec!["../test_data/mbr/mbr.raw", "path", "/test"],
        vec!["../test_data/mbr/mbr.raw", "identifier", "1"],
    ];
    for arguments in &error_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(
            !output.status.success(),
            "keramicsinfo unexpectedly succeeded on {:?}",
            arguments
        );
    }
}

#[test]
fn test_keramicsimage_commands() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsimage");

    let success_cases: Vec<Vec<&str>> = vec![
        // Test hierarchy subcommand.
        vec!["../test_data/bsdlabel/bsdlabel.raw", "hierarchy"],
        vec!["../test_data/gpt/gpt.raw", "hierarchy"],
        vec!["../test_data/mbr/mbr.raw", "hierarchy"],
        vec!["../test_data/apfs/apfs.raw", "hierarchy"],
        vec!["../test_data/apfs/apfs_encrypted.raw", "hierarchy"],
        vec!["../test_data/linuxlvm/lvm2.raw", "hierarchy"],
        vec![
            "../test_data/pdi/hfsplus.hdd/DiskDescriptor.xml",
            "hierarchy",
        ],
        vec!["../test_data/vhd/ntfs-differential.vhd", "hierarchy"],
        vec!["../test_data/vhdx/ntfs-differential.vhdx", "hierarchy"],
        vec!["../test_data/volsnap/volsnap.vhd", "hierarchy"],
        // Test bodyfile subcommand across file system types and options.
        vec!["../test_data/ext/ext2.raw", "bodyfile"],
        vec!["../test_data/ext/ext2.raw", "bodyfile", "--calculate-md5"],
        vec![
            "../test_data/ntfs/ntfs.raw",
            "bodyfile",
            "--output-format",
            "dfimagetools",
            "--calculate-md5",
        ],
        vec!["../test_data/apfs/apfs.raw", "bodyfile"],
        vec!["../test_data/exfat/exfat.raw", "bodyfile"],
        vec!["../test_data/fat/fat12.raw", "bodyfile"],
        vec!["../test_data/hfs/hfsplus.raw", "bodyfile"],
        vec!["../test_data/xfs/xfs.raw", "bodyfile"],
        vec![
            "../test_data/gpt/gpt.raw",
            "bodyfile",
            "--partitions",
            "1",
            "--volume-path-type",
            "identifier",
        ],
        vec![
            "../test_data/linuxlvm/lvm2.raw",
            "bodyfile",
            "--volumes",
            "1",
        ],
        vec![
            "../test_data/ext/ext2.raw",
            "bodyfile",
            "--output-format",
            "dfimagetools",
        ],
        vec!["../test_data/volsnap/volsnap.vhd", "bodyfile"],
        vec!["../test_data/directory/file.txt", "bodyfile"],
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "--credential",
            "recovery_password:000000-000000-000000-000000-000000-000000-000000-000000",
            "../test_data/ext/ext2.raw",
            "hierarchy",
        ],
        // Test hash subcommand on EWF and VHD images.
        vec!["../test_data/ewf/ext2.E01", "hash"],
        vec!["../test_data/vhd/ext2.vhd", "hash"],
    ];
    for arguments in &success_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(
            output.status.success(),
            "keramicsimage failed on {:?}: stdout={} stderr={}",
            arguments,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test error cases.
    let error_cases: Vec<Vec<&str>> = vec![
        vec!["../test_data/nonexistent.raw", "hierarchy"],
        vec!["../test_data/nonexistent.raw", "bodyfile"],
        vec!["../test_data/nonexistent.raw", "hash"],
        vec!["--credential", "bad", "../test_data/ext/ext2.raw"],
        vec!["--credential", "unknown:x", "../test_data/ext/ext2.raw"],
        vec![
            "../test_data/gpt/gpt.raw",
            "bodyfile",
            "--partitions",
            "none",
        ],
        vec![
            "../test_data/linuxlvm/lvm2.raw",
            "bodyfile",
            "--volumes",
            "none",
        ],
    ];
    for arguments in &error_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(
            !output.status.success(),
            "keramicsimage unexpectedly succeeded on {:?}",
            arguments
        );
    }
}

#[test]
fn test_keramicshash_commands() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicshash");

    // Test hashing from standard input.
    let mut child: Child = Command::new(binary_path)
        .args(["--digest-hash-type", "sha256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn keramicshash");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"keramics")
        .unwrap();
    let stdin_output: Output = child.wait_with_output().unwrap();
    assert!(stdin_output.status.success());

    let success_cases: Vec<Vec<&str>> = vec![
        vec!["../test_data/directory/file.txt"],
        vec!["../test_data/ext/ext2.raw"],
        vec![
            "--extended-attributes",
            "../test_data/hfs/hfsplus.raw",
            "hierarchy",
        ],
        vec![
            "--output-format",
            "dfimagetools",
            "../test_data/ntfs/ntfs.raw",
            "hierarchy",
        ],
        vec!["../test_data/ext/ext2.raw", "path", "/testdir1/testfile1"],
        vec![
            "--partitions",
            "1",
            "--volume-path-type",
            "identifier",
            "../test_data/gpt/gpt.raw",
            "path",
            "/testdir1/testfile1",
        ],
        vec![
            "--credential",
            "passphrase:KeRaMiCs",
            "--credential",
            "recovery_password:000000-000000-000000-000000-000000-000000-000000-000000",
            "../test_data/qcow/ext2_aes128.qcow2",
            "path",
            "/testdir1/testfile1",
        ],
        vec!["../test_data/volsnap/volsnap.vhd", "hierarchy"],
        vec![
            "../test_data/ntfs/ntfs.raw",
            "path",
            "/$UpCase",
            "--name",
            "$Info",
        ],
    ];
    for arguments in &success_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(
            output.status.success(),
            "keramicshash failed on {:?}: stdout={} stderr={}",
            arguments,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let error_cases: Vec<Vec<&str>> = vec![
        vec!["../test_data/nonexistent.raw"],
        vec!["../test_data/directory/file.txt", "path", "/file"],
        vec!["--credential", "bad", "../test_data/directory/file.txt"],
        vec![
            "--credential",
            "unknown:x",
            "../test_data/directory/file.txt",
        ],
    ];
    for arguments in &error_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(!output.status.success());
    }
}

#[test]
fn test_keramicsexport_commands() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsexport");

    let mut target_directory: PathBuf = env::temp_dir();
    target_directory.push(format!("keramics_export_test_{}", process::id()));
    _ = fs::remove_dir_all(&target_directory);
    let target_string: &str = target_directory.to_str().unwrap();

    // Test exporting an existing file from an ext2 file system.
    let output: Output = run_binary(
        binary_path,
        &[
            "--target",
            target_string,
            "../test_data/ext/ext2.raw",
            "path",
            "/testdir1/testfile1",
        ],
    );
    assert!(
        output.status.success(),
        "keramicsexport failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let exported_file: PathBuf = target_directory.join("ext2.raw.export").join("testfile1");
    assert!(exported_file.exists());
    assert_eq!(fs::read(&exported_file).unwrap(), b"Keramics\n");

    // Test exporting from a GPT image with the partitions option.
    let output: Output = run_binary(
        binary_path,
        &[
            "--target",
            target_string,
            "--partitions",
            "1",
            "../test_data/gpt/gpt.raw",
            "path",
            "/testdir1/testfile1",
        ],
    );
    assert!(output.status.success());

    // Test exporting a nonexistent path inside a valid file system.
    let output: Output = run_binary(
        binary_path,
        &[
            "--target",
            target_string,
            "../test_data/ext/ext2.raw",
            "path",
            "/does_not_exist",
        ],
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("No data streams exported"));

    // Test error cases.
    for arguments in [
        vec!["../test_data/nonexistent.raw", "path", "/test"],
        vec!["../test_data/directory/file.txt", "path", "/test"],
        vec![
            "--partitions",
            "none",
            "../test_data/gpt/gpt.raw",
            "path",
            "/test",
        ],
        vec![
            "--volumes",
            "none",
            "../test_data/linuxlvm/lvm2.raw",
            "path",
            "/test",
        ],
        vec!["/", "path", "/test"],
    ] {
        let output: Output = run_binary(binary_path, &arguments);
        assert!(
            !output.status.success(),
            "keramicsexport unexpectedly succeeded on {:?}",
            arguments
        );
    }

    _ = fs::remove_dir_all(&target_directory);
}

#[test]
fn test_keramicsutil_commands() {
    let binary_path: &str = env!("CARGO_BIN_EXE_keramicsutil");

    let success_cases: Vec<Vec<&str>> = vec![
        vec!["../test_data/directory/file.txt", "checksum", "adler32"],
        vec![
            "--offset",
            "1",
            "--size",
            "4",
            "../test_data/directory/file.txt",
            "checksum",
            "crc32",
        ],
        vec!["../test_data/directory/file.txt", "checksum", "fletcher64"],
        vec!["../test_data/adc/compressed", "decompress", "adc"],
        vec!["../test_data/lzvn/compressed", "decompress", "lzvn"],
    ];
    for arguments in &success_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(
            output.status.success(),
            "keramicsutil failed on {:?}: stdout={} stderr={}",
            arguments,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let error_cases: Vec<Vec<&str>> = vec![
        vec!["../test_data/nonexistent.bin", "checksum", "crc32"],
        vec![
            "--size",
            "20000000",
            "../test_data/directory/file.txt",
            "checksum",
            "crc32",
        ],
        vec![
            "--size",
            "4",
            "../test_data/nonexistent.bin",
            "checksum",
            "crc32",
        ],
        vec![
            "--size",
            "4",
            "../test_data/nonexistent.bin",
            "decompress",
            "adc",
        ],
        vec!["../test_data/directory/file.txt", "decompress", "bzip2"],
    ];
    for arguments in &error_cases {
        let output: Output = run_binary(binary_path, arguments);
        assert!(!output.status.success());
    }
}
