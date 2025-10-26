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

use keramics_core::ErrorTrace;

use super::ascii::DecoderAscii;
use super::enums::CharacterEncoding;
use super::iso8859_1::DecoderIso8859_1;
use super::iso8859_2::DecoderIso8859_2;
use super::iso8859_3::DecoderIso8859_3;
use super::iso8859_4::DecoderIso8859_4;
use super::iso8859_5::DecoderIso8859_5;
use super::iso8859_6::DecoderIso8859_6;
use super::iso8859_7::DecoderIso8859_7;
use super::iso8859_8::DecoderIso8859_8;
use super::iso8859_9::DecoderIso8859_9;
use super::iso8859_10::DecoderIso8859_10;
use super::iso8859_11::DecoderIso8859_11;
use super::iso8859_13::DecoderIso8859_13;
use super::iso8859_14::DecoderIso8859_14;
use super::iso8859_15::DecoderIso8859_15;
use super::iso8859_16::DecoderIso8859_16;
use super::koi8_r::DecoderKoi8R;
use super::koi8_u::DecoderKoi8U;
use super::mac_arabic::DecoderMacArabic;
use super::mac_celtic::DecoderMacCeltic;
use super::mac_central_eur_roman::DecoderMacCentralEurRoman;
use super::mac_croatian::DecoderMacCroatian;
use super::mac_cyrillic::DecoderMacCyrillic;
use super::mac_dingbats::DecoderMacDingbats;
use super::mac_farsi::DecoderMacFarsi;
use super::mac_gaelic::DecoderMacGaelic;
use super::utf8::DecoderUtf8;
use super::windows874::DecoderWindows874;
use super::windows932::DecoderWindows932;
use super::windows1250::DecoderWindows1250;
use super::windows1251::DecoderWindows1251;
use super::windows1252::DecoderWindows1252;
use super::windows1253::DecoderWindows1253;
use super::windows1254::DecoderWindows1254;
use super::windows1255::DecoderWindows1255;
use super::windows1256::DecoderWindows1256;
use super::windows1257::DecoderWindows1257;
use super::windows1258::DecoderWindows1258;

pub type CharacterDecoder<'a> = Box<dyn Iterator<Item = Result<u32, ErrorTrace>> + 'a>;

/// Creates a new character decoder.
pub fn new_character_decoder<'a>(
    encoding: &CharacterEncoding,
    bytes: &'a [u8],
) -> CharacterDecoder<'a> {
    match encoding {
        CharacterEncoding::Ascii => Box::new(DecoderAscii::new(bytes)),
        CharacterEncoding::Iso8859_1 => Box::new(DecoderIso8859_1::new(bytes)),
        CharacterEncoding::Iso8859_2 => Box::new(DecoderIso8859_2::new(bytes)),
        CharacterEncoding::Iso8859_3 => Box::new(DecoderIso8859_3::new(bytes)),
        CharacterEncoding::Iso8859_4 => Box::new(DecoderIso8859_4::new(bytes)),
        CharacterEncoding::Iso8859_5 => Box::new(DecoderIso8859_5::new(bytes)),
        CharacterEncoding::Iso8859_6 => Box::new(DecoderIso8859_6::new(bytes)),
        CharacterEncoding::Iso8859_7 => Box::new(DecoderIso8859_7::new(bytes)),
        CharacterEncoding::Iso8859_8 => Box::new(DecoderIso8859_8::new(bytes)),
        CharacterEncoding::Iso8859_9 => Box::new(DecoderIso8859_9::new(bytes)),
        CharacterEncoding::Iso8859_10 => Box::new(DecoderIso8859_10::new(bytes)),
        CharacterEncoding::Iso8859_11 => Box::new(DecoderIso8859_11::new(bytes)),
        CharacterEncoding::Iso8859_13 => Box::new(DecoderIso8859_13::new(bytes)),
        CharacterEncoding::Iso8859_14 => Box::new(DecoderIso8859_14::new(bytes)),
        CharacterEncoding::Iso8859_15 => Box::new(DecoderIso8859_15::new(bytes)),
        CharacterEncoding::Iso8859_16 => Box::new(DecoderIso8859_16::new(bytes)),
        CharacterEncoding::Koi8R => Box::new(DecoderKoi8R::new(bytes)),
        CharacterEncoding::Koi8U => Box::new(DecoderKoi8U::new(bytes)),
        CharacterEncoding::MacArabic => Box::new(DecoderMacArabic::new(bytes)),
        CharacterEncoding::MacCeltic => Box::new(DecoderMacCeltic::new(bytes)),
        CharacterEncoding::MacCentralEurRoman => Box::new(DecoderMacCentralEurRoman::new(bytes)),
        CharacterEncoding::MacCroatian => Box::new(DecoderMacCroatian::new(bytes)),
        CharacterEncoding::MacCyrillic => Box::new(DecoderMacCyrillic::new(bytes)),
        CharacterEncoding::MacDingbats => Box::new(DecoderMacDingbats::new(bytes)),
        CharacterEncoding::MacFarsi => Box::new(DecoderMacFarsi::new(bytes)),
        CharacterEncoding::MacGaelic => Box::new(DecoderMacGaelic::new(bytes)),
        CharacterEncoding::Utf8 => Box::new(DecoderUtf8::new(bytes)),
        CharacterEncoding::Windows874 => Box::new(DecoderWindows874::new(bytes)),
        CharacterEncoding::Windows932 => Box::new(DecoderWindows932::new(bytes)),
        CharacterEncoding::Windows1250 => Box::new(DecoderWindows1250::new(bytes)),
        CharacterEncoding::Windows1251 => Box::new(DecoderWindows1251::new(bytes)),
        CharacterEncoding::Windows1252 => Box::new(DecoderWindows1252::new(bytes)),
        CharacterEncoding::Windows1253 => Box::new(DecoderWindows1253::new(bytes)),
        CharacterEncoding::Windows1254 => Box::new(DecoderWindows1254::new(bytes)),
        CharacterEncoding::Windows1255 => Box::new(DecoderWindows1255::new(bytes)),
        CharacterEncoding::Windows1256 => Box::new(DecoderWindows1256::new(bytes)),
        CharacterEncoding::Windows1257 => Box::new(DecoderWindows1257::new(bytes)),
        CharacterEncoding::Windows1258 => Box::new(DecoderWindows1258::new(bytes)),
        _ => todo!(),
    }
}
