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

/// Format identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FormatIdentifier {
    Apm,
    Ext,
    Ewf,
    Fat,
    Gpt,
    Mbr,
    Ntfs,
    Qcow,
    SparseImage,
    SplitRaw,
    Udif,
    Unknown,
    Vhd,
    Vhdx,
}

impl fmt::Display for FormatIdentifier {
    /// Formats the format identifier for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: &str = match self {
            FormatIdentifier::Apm => "apm",
            FormatIdentifier::Ext => "ext",
            FormatIdentifier::Ewf => "ewf",
            FormatIdentifier::Fat => "fat",
            FormatIdentifier::Gpt => "gpt",
            FormatIdentifier::Mbr => "mbr",
            FormatIdentifier::Ntfs => "ntfs",
            FormatIdentifier::Qcow => "qcow",
            FormatIdentifier::SparseImage => "sparseimage",
            FormatIdentifier::SplitRaw => "splitraw",
            FormatIdentifier::Udif => "udif",
            FormatIdentifier::Unknown => "unknown",
            FormatIdentifier::Vhd => "vhd",
            FormatIdentifier::Vhdx => "vhdx",
        };
        write!(formatter, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_identifier_fmt() {
        let format_identifier: FormatIdentifier = FormatIdentifier::Apm;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "apm");

        let format_identifier: FormatIdentifier = FormatIdentifier::Ext;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "ext");

        let format_identifier: FormatIdentifier = FormatIdentifier::Ewf;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "ewf");

        let format_identifier: FormatIdentifier = FormatIdentifier::Fat;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "fat");

        let format_identifier: FormatIdentifier = FormatIdentifier::Gpt;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "gpt");

        let format_identifier: FormatIdentifier = FormatIdentifier::Mbr;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "mbr");

        let format_identifier: FormatIdentifier = FormatIdentifier::Ntfs;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "ntfs");

        let format_identifier: FormatIdentifier = FormatIdentifier::Qcow;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "qcow");

        let format_identifier: FormatIdentifier = FormatIdentifier::SparseImage;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "sparseimage");

        let format_identifier: FormatIdentifier = FormatIdentifier::SplitRaw;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "splitraw");

        let format_identifier: FormatIdentifier = FormatIdentifier::Udif;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "udif");

        let format_identifier: FormatIdentifier = FormatIdentifier::Unknown;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "unknown");

        let format_identifier: FormatIdentifier = FormatIdentifier::Vhd;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "vhd");

        let format_identifier: FormatIdentifier = FormatIdentifier::Vhdx;
        let string: String = format_identifier.to_string();
        assert_eq!(string, "vhdx");
    }
}
