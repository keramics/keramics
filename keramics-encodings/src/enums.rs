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

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum CharacterEncoding {
    Ascii,
    Iso8859_1,
    Iso8859_2,
    Iso8859_3,
    Iso8859_4,
    Iso8859_5,
    Iso8859_6,
    Iso8859_7,
    Iso8859_8,
    Iso8859_9,
    Iso8859_10,
    Iso8859_11,
    // ISO 8859-12 was proposed but never formalized.
    Iso8859_13,
    Iso8859_14,
    Iso8859_15,
    Iso8859_16,
    Koi8R,
    Koi8U,
    MacArabic,
    MacCentic,
    MacCentralEurRoman,
    MacChineseSimplified,
    MacChineseTraditional,
    MacCroatian,
    MacCyrillic,
    MacDingbats,
    MacFarsi,
    MacGaelic,
    MacGreek,
    MacHebrew,
    MacIcelandic,
    MacInuit,
    MacJapanese,
    MacKorean,
    MacRoman,
    MacRussian,
    MacSymbol,
    MacThai,
    MacTurkish,
    MacUkrainian,
    Ucs2,
    Utf16BigEndian,
    Utf16LittleEndian,
    Utf32BigEndian,
    Utf32LittleEndian,
    Utf8,
    Windows874,
    Windows932,
    Windows936,
    Windows949,
    Windows950,
    Windows1250,
    Windows1251,
    Windows1252,
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
}
