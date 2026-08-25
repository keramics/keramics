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

mod apfs;
mod apm;
mod bsdlabel;
mod cdsaencr;
mod constants;
mod ewf;
mod exfat;
mod ext;
mod fat;
mod gpt;
mod hfs;
mod linuxlvm;
mod luksde;
mod mbr;
mod ntfs;
mod pdi;
mod posix;
mod qcow;
mod sparsebundle;
mod sparseimage;
mod udif;
mod vhd;
mod vhdx;
mod vmdk;
mod windows;

pub use apfs::ApfsInfo;
pub use apm::ApmInfo;
pub use bsdlabel::BsdDiskLabelInfo;
pub use cdsaencr::CdsaEncrInfo;
pub use ewf::EwfInfo;
pub use exfat::ExFatInfo;
pub use ext::ExtInfo;
pub use fat::FatInfo;
pub use gpt::GptInfo;
pub use hfs::HfsInfo;
pub use linuxlvm::LinuxLvmInfo;
pub use luksde::LuksInfo;
pub use mbr::MbrInfo;
pub use ntfs::NtfsInfo;
pub use pdi::PdiInfo;
pub use qcow::QcowInfo;
pub use sparsebundle::SparseBundleInfo;
pub use sparseimage::SparseImageInfo;
pub use udif::UdifInfo;
pub use vhd::VhdInfo;
pub use vhdx::VhdxInfo;
pub use vmdk::VmdkInfo;

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! assert_lines_eq {
        ( $text:expr, $expected_text:expr $(,)? ) => {
            let mut lines = $text.lines();
            let mut expected_lines = $expected_text.lines();

            for (line_index, (line, expected_line)) in
                lines.by_ref().zip(expected_lines.by_ref()).enumerate()
            {
                assert_eq!(
                    line,
                    expected_line,
                    "line: {} does not match",
                    line_index + 1
                );
            }
            assert_eq!(lines.next(), None, "additional lines");
            assert_eq!(expected_lines.next(), None, "missing lines");
        };
    }
}
