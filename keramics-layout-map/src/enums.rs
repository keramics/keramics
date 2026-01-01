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

#[derive(Clone, PartialEq)]
pub enum BitOrder {
    MostSignificantBit,
    LeastSignificantBit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
    NotSet,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    BitField8,
    BitField16,
    BitField32,
    BitField64,
    ByteString,
    FatDate,
    FatTimeDate,
    FatTimeDate10Ms,
    Filetime,
    HfsTime,
    NotSet,
    PosixTime32,
    SignedInteger8Bit,
    SignedInteger16Bit,
    SignedInteger32Bit,
    SignedInteger64Bit,
    Struct { name: String, size: usize },
    Ucs2String,
    Utf16String,
    UnsignedInteger8Bit,
    UnsignedInteger16Bit,
    UnsignedInteger32Bit,
    UnsignedInteger64Bit,
    Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Format {
    Character,
    Hexadecimal,
    NotSet,
}
