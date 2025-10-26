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

use clap::ValueEnum;

/// Digest hash types.
#[derive(Clone, ValueEnum)]
pub enum DigestHashType {
    /// MD5
    Md5,

    /// SHA1
    Sha1,

    /// SHA-224
    Sha224,

    /// SHA-256
    Sha256,

    /// SHA-512
    Sha512,
}

/// Display path types.
#[derive(Clone, Debug, ValueEnum)]
pub enum DisplayPathType {
    /// Identifier based volume or partition path, such as /apfs{f449e580-e355-4e74-8880-05e46e4e3b1e}
    Identifier,

    /// Index based volume or partition path, such as /apfs1 or /p1
    Index,
}

/// Encoding types.
#[derive(Clone, ValueEnum)]
pub enum EncodingType {
    /// ASCII
    Ascii,

    /// ISO 8859-1 (Western European)
    #[value(alias("latin-1"))]
    Iso8859_1,

    /// ISO 8859-2 (Central European)
    #[value(alias("latin-2"))]
    Iso8859_2,

    /// ISO 8859-3 (South European)
    #[value(alias("latin-3"))]
    Iso8859_3,

    /// ISO 8859-4 (North European)
    #[value(alias("latin-4"))]
    Iso8859_4,

    /// ISO 8859-5 (Cyrillic)
    Iso8859_5,

    /// ISO 8859-6 (Arabic)
    Iso8859_6,

    /// ISO 8859-7 (Greek)
    Iso8859_7,

    /// ISO 8859-8 (Hebrew)
    Iso8859_8,

    /// ISO 8859-9 (Turkish)
    #[value(alias("latin-5"))]
    Iso8859_9,

    /// ISO 8859-10 (Nordic)
    #[value(alias("latin-6"))]
    Iso8859_10,

    /// ISO 8859-11 (Thai)
    Iso8859_11,

    /// ISO 8859-13 (Baltic Rim)
    #[value(alias("latin-7"))]
    Iso8859_13,

    /// ISO 8859-14 (Celtic)
    #[value(alias("latin-8"))]
    Iso8859_14,

    /// ISO 8859-15
    #[value(alias("latin-9"))]
    Iso8859_15,

    /// ISO 8859-16 (South-Eastern European)
    #[value(alias("latin-10"))]
    Iso8859_16,

    /// UTF-8
    Utf8,
}
