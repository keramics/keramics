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

use super::ascii::EncoderAscii;
use super::enums::CharacterEncoding;
use super::iso8859_1::EncoderIso8859_1;
use super::iso8859_2::EncoderIso8859_2;
use super::iso8859_3::EncoderIso8859_3;
use super::iso8859_4::EncoderIso8859_4;
use super::iso8859_5::EncoderIso8859_5;
use super::iso8859_6::EncoderIso8859_6;
use super::iso8859_7::EncoderIso8859_7;
use super::iso8859_8::EncoderIso8859_8;
use super::iso8859_9::EncoderIso8859_9;
use super::iso8859_10::EncoderIso8859_10;
use super::iso8859_11::EncoderIso8859_11;
use super::iso8859_13::EncoderIso8859_13;
use super::iso8859_14::EncoderIso8859_14;
use super::iso8859_15::EncoderIso8859_15;
use super::iso8859_16::EncoderIso8859_16;
use super::koi8_r::EncoderKoi8R;
use super::koi8_u::EncoderKoi8U;
use super::mac_arabic::EncoderMacArabic;
use super::mac_celtic::EncoderMacCeltic;
use super::mac_central_eur_roman::EncoderMacCentralEurRoman;
use super::mac_croatian::EncoderMacCroatian;
use super::mac_cyrillic::EncoderMacCyrillic;
use super::mac_dingbats::EncoderMacDingbats;
use super::mac_farsi::EncoderMacFarsi;
use super::mac_gaelic::EncoderMacGaelic;
use super::windows874::EncoderWindows874;
use super::windows932::EncoderWindows932;
use super::windows1250::EncoderWindows1250;
use super::windows1251::EncoderWindows1251;
use super::windows1252::EncoderWindows1252;
use super::windows1253::EncoderWindows1253;
use super::windows1254::EncoderWindows1254;
use super::windows1255::EncoderWindows1255;
use super::windows1256::EncoderWindows1256;
use super::windows1257::EncoderWindows1257;
use super::windows1258::EncoderWindows1258;

pub type CharacterEncoder<'a> = Box<dyn Iterator<Item = Result<Vec<u8>, ErrorTrace>> + 'a>;

/// Creates a new character encoder.
pub fn new_character_encoder<'a>(
    encoding: &CharacterEncoding,
    code_points: &'a [u32],
) -> CharacterEncoder<'a> {
    match encoding {
        CharacterEncoding::Ascii => Box::new(EncoderAscii::new(code_points)),
        CharacterEncoding::Iso8859_1 => Box::new(EncoderIso8859_1::new(code_points)),
        CharacterEncoding::Iso8859_2 => Box::new(EncoderIso8859_2::new(code_points)),
        CharacterEncoding::Iso8859_3 => Box::new(EncoderIso8859_3::new(code_points)),
        CharacterEncoding::Iso8859_4 => Box::new(EncoderIso8859_4::new(code_points)),
        CharacterEncoding::Iso8859_5 => Box::new(EncoderIso8859_5::new(code_points)),
        CharacterEncoding::Iso8859_6 => Box::new(EncoderIso8859_6::new(code_points)),
        CharacterEncoding::Iso8859_7 => Box::new(EncoderIso8859_7::new(code_points)),
        CharacterEncoding::Iso8859_8 => Box::new(EncoderIso8859_8::new(code_points)),
        CharacterEncoding::Iso8859_9 => Box::new(EncoderIso8859_9::new(code_points)),
        CharacterEncoding::Iso8859_10 => Box::new(EncoderIso8859_10::new(code_points)),
        CharacterEncoding::Iso8859_11 => Box::new(EncoderIso8859_11::new(code_points)),
        CharacterEncoding::Iso8859_13 => Box::new(EncoderIso8859_13::new(code_points)),
        CharacterEncoding::Iso8859_14 => Box::new(EncoderIso8859_14::new(code_points)),
        CharacterEncoding::Iso8859_15 => Box::new(EncoderIso8859_15::new(code_points)),
        CharacterEncoding::Iso8859_16 => Box::new(EncoderIso8859_16::new(code_points)),
        CharacterEncoding::Koi8R => Box::new(EncoderKoi8R::new(code_points)),
        CharacterEncoding::Koi8U => Box::new(EncoderKoi8U::new(code_points)),
        CharacterEncoding::MacArabic => Box::new(EncoderMacArabic::new(code_points)),
        CharacterEncoding::MacCeltic => Box::new(EncoderMacCeltic::new(code_points)),
        CharacterEncoding::MacCentralEurRoman => {
            Box::new(EncoderMacCentralEurRoman::new(code_points))
        }
        CharacterEncoding::MacCroatian => Box::new(EncoderMacCroatian::new(code_points)),
        CharacterEncoding::MacCyrillic => Box::new(EncoderMacCyrillic::new(code_points)),
        CharacterEncoding::MacDingbats => Box::new(EncoderMacDingbats::new(code_points)),
        CharacterEncoding::MacFarsi => Box::new(EncoderMacFarsi::new(code_points)),
        CharacterEncoding::MacGaelic => Box::new(EncoderMacGaelic::new(code_points)),
        CharacterEncoding::Windows874 => Box::new(EncoderWindows874::new(code_points)),
        CharacterEncoding::Windows932 => Box::new(EncoderWindows932::new(code_points)),
        CharacterEncoding::Windows1250 => Box::new(EncoderWindows1250::new(code_points)),
        CharacterEncoding::Windows1251 => Box::new(EncoderWindows1251::new(code_points)),
        CharacterEncoding::Windows1252 => Box::new(EncoderWindows1252::new(code_points)),
        CharacterEncoding::Windows1253 => Box::new(EncoderWindows1253::new(code_points)),
        CharacterEncoding::Windows1254 => Box::new(EncoderWindows1254::new(code_points)),
        CharacterEncoding::Windows1255 => Box::new(EncoderWindows1255::new(code_points)),
        CharacterEncoding::Windows1256 => Box::new(EncoderWindows1256::new(code_points)),
        CharacterEncoding::Windows1257 => Box::new(EncoderWindows1257::new(code_points)),
        CharacterEncoding::Windows1258 => Box::new(EncoderWindows1258::new(code_points)),
        _ => todo!(),
    }
}
