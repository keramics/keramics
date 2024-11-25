/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

#[derive(Clone, Default, PartialEq)]
pub enum BitOrder {
    MostSignificantBit,
    LeastSignificantBit,
    #[default]
    NotSet,
}

#[derive(Clone, Default, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
    #[default]
    NotSet,
}

#[derive(Default, PartialEq)]
pub enum DataType {
    BitField16,
    BitField32,
    BitField64,
    ByteString,
    #[default]
    NotSet,
    SignedInteger8Bit,
    SignedInteger16Bit,
    SignedInteger32Bit,
    SignedInteger64Bit,
    Struct {
        name: String,
        size: usize,
    },
    Ucs2String,
    Utf16String,
    UnsignedInteger8Bit,
    UnsignedInteger16Bit,
    UnsignedInteger32Bit,
    UnsignedInteger64Bit,
    Uuid,
}

impl Clone for DataType {
    fn clone(&self) -> DataType {
        match self {
            DataType::BitField16 => DataType::BitField16,
            DataType::BitField32 => DataType::BitField32,
            DataType::BitField64 => DataType::BitField64,
            DataType::ByteString => DataType::ByteString,
            DataType::NotSet => DataType::NotSet,
            DataType::SignedInteger8Bit => DataType::SignedInteger8Bit,
            DataType::SignedInteger16Bit => DataType::SignedInteger16Bit,
            DataType::SignedInteger32Bit => DataType::SignedInteger32Bit,
            DataType::SignedInteger64Bit => DataType::SignedInteger64Bit,
            DataType::Struct { name, size } => DataType::Struct {
                name: name.to_string(),
                size: *size,
            },

            DataType::Ucs2String => DataType::Ucs2String,
            DataType::Utf16String => DataType::Utf16String,
            DataType::UnsignedInteger8Bit => DataType::UnsignedInteger8Bit,
            DataType::UnsignedInteger16Bit => DataType::UnsignedInteger16Bit,
            DataType::UnsignedInteger32Bit => DataType::UnsignedInteger32Bit,
            DataType::UnsignedInteger64Bit => DataType::UnsignedInteger64Bit,
            DataType::Uuid => DataType::Uuid,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub enum Format {
    Character,
    Hexadecimal,
    #[default]
    NotSet,
}
