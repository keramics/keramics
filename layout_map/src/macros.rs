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

use darling::{FromDeriveInput, FromMeta};

use quote::quote;

use crate::bitmap::BitmapLayout;
use crate::enums::{BitOrder, ByteOrder, DataType, Format};
use crate::errors::ParseError;
use crate::structure::{StructureLayout, StructureLayoutField};

// TODO: add ondemand vector type

#[derive(Default, FromMeta)]
#[darling(default)]
struct FieldOptions {
    /// Byte order.
    byte_order: String,

    /// Data type.
    data_type: String,

    /// Format.
    format: String,

    /// Modifier.
    modifier: String,

    /// Name.
    name: String,
}

impl FieldOptions {
    /// Parses the byte order.
    pub fn parse_byte_order(&self) -> Result<ByteOrder, ParseError> {
        match self.byte_order.as_str() {
            "be" | "big" | "BigEndian" => Ok(ByteOrder::BigEndian),
            "le" | "little" | "LittleEndian" => Ok(ByteOrder::LittleEndian),
            "" => Ok(ByteOrder::NotSet),
            _ => {
                return Err(ParseError::new(format!(
                    "Unsupported byte order: {}",
                    self.byte_order
                )))
            }
        }
    }

    /// Parses the data type.
    fn parse_data_type(&self) -> Result<(DataType, usize), ParseError> {
        if self.data_type.is_empty() {
            panic!("Data type missing")
        }
        let mut data_type_str: &str = self.data_type.as_str();
        let mut number_of_elements_str: &str = "";

        if data_type_str.starts_with("[") && data_type_str.ends_with("]") {
            data_type_str = data_type_str.strip_prefix("[").unwrap();
            data_type_str = data_type_str.strip_suffix("]").unwrap();
            (data_type_str, number_of_elements_str) = data_type_str.rsplit_once(";").unwrap();
            data_type_str = data_type_str.trim();
            number_of_elements_str = number_of_elements_str.trim();
        }
        let data_type: DataType = match data_type_str {
            "i8" | "int8" | "SignedInteger8Bit" => DataType::SignedInteger8Bit,
            "i16" | "int16" | "SignedInteger16Bit" => DataType::SignedInteger16Bit,
            "i32" | "int32" | "SignedInteger32Bit" => DataType::SignedInteger32Bit,
            "i64" | "int64" | "SignedInteger64Bit" => DataType::SignedInteger64Bit,
            "u8" | "uint8" | "UnsignedInteger8Bit" => DataType::UnsignedInteger8Bit,
            "u16" | "uint16" | "UnsignedInteger16Bit" => DataType::UnsignedInteger16Bit,
            "u32" | "uint32" | "UnsignedInteger32Bit" => DataType::UnsignedInteger32Bit,
            "u64" | "uint64" | "UnsignedInteger64Bit" => DataType::UnsignedInteger64Bit,
            "uuid" | "Uuid" => DataType::Uuid,
            _ => {
                if data_type_str.starts_with("BitField16<") && data_type_str.ends_with(">") {
                    data_type_str = data_type_str.strip_prefix("BitField16<").unwrap();
                    number_of_elements_str = data_type_str.strip_suffix(">").unwrap();
                    DataType::BitField16
                } else if data_type_str.starts_with("BitField32<") && data_type_str.ends_with(">") {
                    data_type_str = data_type_str.strip_prefix("BitField32<").unwrap();
                    number_of_elements_str = data_type_str.strip_suffix(">").unwrap();
                    DataType::BitField32
                } else if data_type_str.starts_with("BitField64<") && data_type_str.ends_with(">") {
                    data_type_str = data_type_str.strip_prefix("BitField64<").unwrap();
                    number_of_elements_str = data_type_str.strip_suffix(">").unwrap();
                    DataType::BitField64
                } else if data_type_str.starts_with("ByteString<") && data_type_str.ends_with(">") {
                    data_type_str = data_type_str.strip_prefix("ByteString<").unwrap();
                    number_of_elements_str = data_type_str.strip_suffix(">").unwrap();
                    DataType::ByteString
                } else if data_type_str.starts_with("Struct<") && data_type_str.ends_with(">") {
                    data_type_str = data_type_str.strip_prefix("Struct<").unwrap();
                    data_type_str = data_type_str.strip_suffix(">").unwrap();
                    let (struct_name_str, struct_size_str) = data_type_str.split_once(";").unwrap();
                    DataType::Struct {
                        name: struct_name_str.trim().to_string(),
                        size: struct_size_str.trim().parse::<usize>().unwrap(),
                    }
                } else if data_type_str.starts_with("Ucs2String<") && data_type_str.ends_with(">") {
                    data_type_str = data_type_str.strip_prefix("Ucs2String<").unwrap();
                    number_of_elements_str = data_type_str.strip_suffix(">").unwrap();
                    DataType::Ucs2String
                } else if data_type_str.starts_with("Utf16String<") && data_type_str.ends_with(">")
                {
                    data_type_str = data_type_str.strip_prefix("Utf16String<").unwrap();
                    number_of_elements_str = data_type_str.strip_suffix(">").unwrap();
                    DataType::Utf16String
                } else {
                    return Err(ParseError::new(format!(
                        "Unsupported data type: {}",
                        data_type_str
                    )));
                }
            }
        };
        let mut number_of_elements: usize = 1;

        if !number_of_elements_str.is_empty() {
            number_of_elements = match number_of_elements_str.parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(ParseError::new(format!(
                        "Unsupported number of elements: {} in data type: {}",
                        number_of_elements_str, data_type_str
                    )))
                }
            }
        }
        Ok((data_type, number_of_elements))
    }

    /// Parses the format.
    pub fn parse_format(&self) -> Result<Format, ParseError> {
        match self.format.as_str() {
            "char" | "Character" => Ok(Format::Character),
            "hex" | "Hexadecimal" => Ok(Format::Hexadecimal),
            "" => Ok(Format::NotSet),
            _ => {
                return Err(ParseError::new(format!(
                    "Unsupported format: {}",
                    self.format
                )))
            }
        }
    }
}

#[derive(Default, FromMeta)]
#[darling(default)]
struct BitmapOptions {
    /// Bit order.
    bit_order: String,

    /// Data type.
    data_type: String,
}

impl BitmapOptions {
    /// Determines if the options are empty.
    pub fn is_empty(&self) -> bool {
        return self.bit_order.is_empty() && self.data_type.is_empty();
    }

    /// Parses the bit order.
    pub fn parse_bit_order(&self) -> Result<BitOrder, ParseError> {
        if self.bit_order.is_empty() {
            panic!("Bit order missing")
        }
        match self.bit_order.as_str() {
            "msb" | "most" | "MostSignificantBit" => Ok(BitOrder::MostSignificantBit),
            "lsb" | "least" | "LeastSignificantBit" => Ok(BitOrder::LeastSignificantBit),
            _ => {
                return Err(ParseError::new(format!(
                    "Unsupported bit order: {}",
                    self.bit_order
                )))
            }
        }
    }

    /// Parses the data type.
    fn parse_data_type(&self) -> Result<DataType, ParseError> {
        if self.data_type.is_empty() {
            panic!("Data type missing")
        }
        let data_type: DataType = match self.data_type.as_str() {
            "u8" | "uint8" | "UnsignedInteger8Bit" => DataType::UnsignedInteger8Bit,
            "u16" | "uint16" | "UnsignedInteger16Bit" => DataType::UnsignedInteger16Bit,
            "u32" | "uint32" | "UnsignedInteger32Bit" => DataType::UnsignedInteger32Bit,
            "u64" | "uint64" | "UnsignedInteger64Bit" => DataType::UnsignedInteger64Bit,
            _ => {
                return Err(ParseError::new(format!(
                    "Unsupported data type: {}",
                    self.data_type
                )))
            }
        };
        Ok(data_type)
    }
}
#[derive(Default, FromMeta)]
#[darling(default)]
struct StructureOptions {
    /// Byte order.
    byte_order: String,

    /// Fields.
    #[darling(default, multiple, rename = "field")]
    fields: Vec<FieldOptions>,
}

impl StructureOptions {
    /// Determines if the options are empty.
    pub fn is_empty(&self) -> bool {
        return self.byte_order.is_empty() && self.fields.len() == 0;
    }

    /// Parses the byte order.
    pub fn parse_byte_order(&self) -> Result<ByteOrder, ParseError> {
        match self.byte_order.as_str() {
            "be" | "big" | "BigEndian" => Ok(ByteOrder::BigEndian),
            "le" | "little" | "LittleEndian" => Ok(ByteOrder::LittleEndian),
            "" => Ok(ByteOrder::NotSet),
            _ => {
                return Err(ParseError::new(format!(
                    "Unsupported byte order: {}",
                    self.byte_order
                )))
            }
        }
    }
}

#[derive(Default, FromMeta)]
#[darling(default)]
struct MethodOptions {
    /// Name.
    name: String,
}

#[derive(FromDeriveInput)]
#[darling(attributes(layout_map), supports(struct_named))]
struct LayoutMapOptions {
    /// Bitmap option.
    #[darling(default)]
    pub bitmap: BitmapOptions,

    /// Structure option.
    #[darling(default)]
    pub structure: StructureOptions,

    /// Method option.
    #[darling(default, multiple, rename = "method")]
    methods: Vec<MethodOptions>,
}

/// Parses a bitmap layout.
fn parse_bitmap_layout(
    struct_ident: &syn::Ident,
    _struct_fields: &syn::Fields,
    options: &LayoutMapOptions,
) -> Result<BitmapLayout, ParseError> {
    let name: String = struct_ident.to_string();

    let bit_order: BitOrder = match options.bitmap.parse_bit_order() {
        Ok(bit_order) => bit_order,
        Err(error) => {
            return Err(ParseError::new(format!(
                "{} in layout map of {}",
                error, name
            )))
        }
    };
    let data_type: DataType = match options.bitmap.parse_data_type() {
        Ok(data_type) => data_type,
        Err(error) => {
            return Err(ParseError::new(format!(
                "{} in layout map of {}",
                error, name
            )))
        }
    };
    let bitmap_layout: BitmapLayout = BitmapLayout::new(data_type, bit_order);

    // TODO: add option for value size and byte order
    todo!();

    // Ok(bitmap_layout)
}

/// Parses a structure layout.
fn parse_structure_layout(
    struct_ident: &syn::Ident,
    _struct_fields: &syn::Fields,
    options: &LayoutMapOptions,
) -> Result<StructureLayout, ParseError> {
    let name: String = struct_ident.to_string();

    let byte_order: ByteOrder = match options.structure.parse_byte_order() {
        Ok(byte_order) => byte_order,
        Err(error) => {
            return Err(ParseError::new(format!(
                "{} in layout map of {}",
                error, name
            )))
        }
    };
    let mut structure_layout: StructureLayout = StructureLayout::new(&name, byte_order);

    for field_options in options.structure.fields.iter() {
        if field_options.name.is_empty() {
            return Err(ParseError::new(format!(
                "Name missing in field in layout map of {}",
                name
            )));
        }
        let (data_type, number_of_elements) = match field_options.parse_data_type() {
            Ok(value) => value,
            Err(error) => {
                return Err(ParseError::new(format!(
                    "{} in field: {}",
                    error, field_options.name
                )))
            }
        };
        let byte_order: ByteOrder = match field_options.parse_byte_order() {
            Ok(byte_order) => byte_order,
            Err(error) => {
                return Err(ParseError::new(format!(
                    "{} in field: {}",
                    error, field_options.name
                )))
            }
        };
        let format: Format = match field_options.parse_format() {
            Ok(format) => format,
            Err(error) => {
                return Err(ParseError::new(format!(
                    "{} in field: {}",
                    error, field_options.name
                )))
            }
        };
        structure_layout.fields.push(StructureLayoutField::new(
            &field_options.name,
            data_type,
            byte_order,
            number_of_elements,
            &field_options.modifier,
            format,
        ));
    }
    Ok(structure_layout)
}

pub fn process_input(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_struct = syn::parse_macro_input!(input as syn::DeriveInput);

    let options: LayoutMapOptions = match LayoutMapOptions::from_derive_input(&input_struct) {
        Ok(options) => options,
        Err(error) => return proc_macro::TokenStream::from(error.write_errors()),
    };
    let syn::DeriveInput { data, ident, .. } = input_struct.clone();

    if let syn::Data::Struct(data_struct) = data {
        let syn::DataStruct { fields, .. } = data_struct;

        if !options.bitmap.is_empty() && !options.structure.is_empty() {
            panic!("LayoutMap cannot combine bitmap and structure definitions");
        }
        let mut methods = quote!();

        if !options.bitmap.is_empty() {
            // TODO: complete bitmap layout support
            let _bitmap_layout: BitmapLayout = match parse_bitmap_layout(&ident, &fields, &options) {
                Ok(bitmap_layout) => bitmap_layout,
                Err(error) => panic!("{error:}"),
            };
        } else if !options.structure.is_empty() {
            let structure_layout: StructureLayout =
                match parse_structure_layout(&ident, &fields, &options) {
                    Ok(structure_layout) => structure_layout,
                    Err(error) => panic!("{error:}"),
                };
            for method_option in options.methods.iter() {
                // TODO: check if read_at_position is used without debug_read_data.
                let generated_code = match method_option.name.as_str() {
                    "debug_read_data" => structure_layout.generate_debug_read_data(),
                    "read_at_position" => structure_layout.generate_read_at_position(),
                    _ => panic!("Unsupported method in layout map of {}", ident),
                };
                methods.extend(generated_code);
            }
        } else {
            panic!("LayoutMap requires a bitmap or structure definition");
        }
        let token_stream = quote! {
            impl #ident {
                #methods
            }
        };
        token_stream.into()
    } else {
        panic!("LayoutMap can only be used with named structs")
    }
}
