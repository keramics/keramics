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

mod ascii;
mod base64;
mod enums;
mod iso8859_1;
mod iso8859_10;
mod iso8859_11;
mod iso8859_13;
mod iso8859_14;
mod iso8859_15;
mod iso8859_16;
mod iso8859_2;
mod iso8859_3;
mod iso8859_4;
mod iso8859_5;
mod iso8859_6;
mod iso8859_7;
mod iso8859_8;
mod iso8859_9;
mod koi8_r;
mod koi8_u;
mod mac_arabic;
mod mac_celtic;
mod mac_central_eur_roman;
mod windows1250;
mod windows1251;
mod windows1252;
mod windows1253;
mod windows1254;
mod windows1255;
mod windows1256;
mod windows1257;
mod windows1258;
mod windows874;
mod windows932;

pub use ascii::{DecoderAscii, EncoderAscii};
pub use base64::{Base64Context, Base64Stream};
pub use enums::CharacterEncoding;
pub use iso8859_1::{DecoderIso8859_1, EncoderIso8859_1};
pub use iso8859_2::{DecoderIso8859_2, EncoderIso8859_2};
pub use iso8859_3::{DecoderIso8859_3, EncoderIso8859_3};
pub use iso8859_4::{DecoderIso8859_4, EncoderIso8859_4};
pub use iso8859_5::{DecoderIso8859_5, EncoderIso8859_5};
pub use iso8859_6::{DecoderIso8859_6, EncoderIso8859_6};
pub use iso8859_7::{DecoderIso8859_7, EncoderIso8859_7};
pub use iso8859_8::{DecoderIso8859_8, EncoderIso8859_8};
pub use iso8859_9::{DecoderIso8859_9, EncoderIso8859_9};
pub use iso8859_10::{DecoderIso8859_10, EncoderIso8859_10};
pub use iso8859_11::{DecoderIso8859_11, EncoderIso8859_11};
pub use iso8859_13::{DecoderIso8859_13, EncoderIso8859_13};
pub use iso8859_14::{DecoderIso8859_14, EncoderIso8859_14};
pub use iso8859_15::{DecoderIso8859_15, EncoderIso8859_15};
pub use iso8859_16::{DecoderIso8859_16, EncoderIso8859_16};
pub use koi8_r::{DecoderKoi8R, EncoderKoi8R};
pub use koi8_u::{DecoderKoi8U, EncoderKoi8U};
pub use mac_arabic::{DecoderMacArabic, EncoderMacArabic};
pub use mac_celtic::{DecoderMacCeltic, EncoderMacCeltic};
pub use mac_central_eur_roman::{DecoderMacCentralEurRoman, EncoderMacCentralEurRoman};
pub use windows874::{DecoderWindows874, EncoderWindows874};
pub use windows932::{DecoderWindows932, EncoderWindows932};
pub use windows1250::{DecoderWindows1250, EncoderWindows1250};
pub use windows1251::{DecoderWindows1251, EncoderWindows1251};
pub use windows1252::{DecoderWindows1252, EncoderWindows1252};
pub use windows1253::{DecoderWindows1253, EncoderWindows1253};
pub use windows1254::{DecoderWindows1254, EncoderWindows1254};
pub use windows1255::{DecoderWindows1255, EncoderWindows1255};
pub use windows1256::{DecoderWindows1256, EncoderWindows1256};
pub use windows1257::{DecoderWindows1257, EncoderWindows1257};
pub use windows1258::{DecoderWindows1258, EncoderWindows1258};
