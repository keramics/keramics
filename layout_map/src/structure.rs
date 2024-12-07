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

use std::slice::Iter;
use std::str::FromStr;

use proc_macro2::{Ident, Literal, TokenStream};

use quote::{format_ident, quote};

use crate::enums::{ByteOrder, DataType, Format};

/// Structure layout field.
pub(crate) struct StructureLayoutField {
    /// Name.
    pub name: String,

    /// Data type.
    pub data_type: DataType,

    /// Byte order.
    pub byte_order: ByteOrder,

    /// Number of elements.
    pub number_of_elements: usize,

    /// Modifier.
    pub modifier: String,

    /// Format.
    pub format: Format,
}

impl StructureLayoutField {
    /// Creates a new field.
    pub fn new(
        name: &String,
        data_type: DataType,
        byte_order: ByteOrder,
        number_of_elements: usize,
        modifier: &String,
        format: Format,
    ) -> Self {
        Self {
            name: name.clone(),
            data_type: data_type,
            byte_order: byte_order,
            number_of_elements: number_of_elements,
            modifier: modifier.clone(),
            format: format,
        }
    }

    /// Retrieves the byte order.
    pub fn get_byte_order(&self, parent_byte_order: &ByteOrder) -> ByteOrder {
        if self.byte_order != ByteOrder::NotSet {
            return self.byte_order.clone();
        }
        if self.requires_byte_order() {
            if parent_byte_order == &ByteOrder::NotSet {
                panic!("Byte order missing in field: {}", self.name);
            }
        }
        parent_byte_order.clone()
    }

    /// Retrieves the byte size.
    pub fn get_byte_size(&self) -> Option<usize> {
        match &self.data_type {
            DataType::BitField16 => Some(2),
            DataType::BitField32 => Some(4),
            DataType::BitField64 => Some(8),
            DataType::ByteString => Some(1),
            DataType::SignedInteger8Bit => Some(1),
            DataType::SignedInteger16Bit => Some(2),
            DataType::SignedInteger32Bit => Some(4),
            DataType::SignedInteger64Bit => Some(8),
            DataType::Struct { size, .. } => Some(*size),
            DataType::Ucs2String => Some(2),
            DataType::Utf16String => Some(2),
            DataType::UnsignedInteger8Bit => Some(1),
            DataType::UnsignedInteger16Bit => Some(2),
            DataType::UnsignedInteger32Bit => Some(4),
            DataType::UnsignedInteger64Bit => Some(8),
            DataType::Uuid => Some(16),
            _ => None,
        }
    }

    /// Retrieves a token stream to convert the field from bytes.
    pub fn get_from_bytes_token_stream(
        &self,
        byte_order: &ByteOrder,
        data_offset: TokenStream,
    ) -> TokenStream {
        match &self.data_type {
            DataType::BitField16 => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u16_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u16_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::BitField32 => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u32_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u32_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::BitField64 => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u64_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u64_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::SignedInteger8Bit | DataType::UnsignedInteger8Bit => {
                quote!(data[#data_offset])
            }
            DataType::SignedInteger16Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_i16_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_i16_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::SignedInteger32Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_i32_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_i32_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::SignedInteger64Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_i64_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_i64_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::UnsignedInteger16Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u16_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u16_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::UnsignedInteger32Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u32_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u32_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::UnsignedInteger64Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u64_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u64_le!(data, #data_offset)),
                _ => todo!(),
            },
            DataType::Uuid => match byte_order {
                &ByteOrder::BigEndian => {
                    quote!(crate::types::Uuid::from_be_bytes(&data[#data_offset..#data_offset + 16]))
                }
                &ByteOrder::LittleEndian => {
                    quote!(crate::types::Uuid::from_le_bytes(&data[#data_offset..#data_offset + 16]))
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    /// Retrieves a token stream of the modifier.
    pub fn get_modifier_token_stream(&self) -> TokenStream {
        match TokenStream::from_str(&self.modifier) {
            Ok(token_stream) => token_stream,
            Err(error) => panic!("Unable to parse modifier: \"{}\" with error: {}", self.modifier, error),
        }
    }

    /// Retrieves a token stream of the type.
    pub fn get_type_token_stream(&self) -> TokenStream {
        match &self.data_type {
            DataType::BitField16 => quote!(u16),
            DataType::BitField32 => quote!(u32),
            DataType::BitField64 => quote!(u64),
            DataType::ByteString => quote!(crate::types::ByteString),
            DataType::SignedInteger8Bit => quote!(i8),
            DataType::SignedInteger16Bit => quote!(i16),
            DataType::SignedInteger32Bit => quote!(i32),
            DataType::SignedInteger64Bit => quote!(i64),
            DataType::Struct { name, .. } => {
                let name_literal: Ident = format_ident!("{}", name);
                quote!(#name_literal)
            }
            DataType::Ucs2String => quote!(crate::types::Ucs2String),
            DataType::Utf16String => quote!(crate::types::Utf16String),
            DataType::UnsignedInteger8Bit => quote!(u8),
            DataType::UnsignedInteger16Bit => quote!(u16),
            DataType::UnsignedInteger32Bit => quote!(u32),
            DataType::UnsignedInteger64Bit => quote!(u64),
            DataType::Uuid => quote!(crate::types::Uuid),
            _ => todo!(),
        }
    }

    /// Determines if field requires a byte order.
    pub fn requires_byte_order(&self) -> bool {
        match self.data_type {
            DataType::SignedInteger8Bit | DataType::UnsignedInteger8Bit => false,
            _ => true,
        }
    }
}

/// Structure layout.
pub(crate) struct StructureLayout {
    /// Name.
    pub name: String,

    /// Byte order.
    pub byte_order: ByteOrder,

    /// Fields.
    pub fields: Vec<StructureLayoutField>,
}

impl StructureLayout {
    /// Creates a new layout.
    pub fn new(name: &String, byte_order: ByteOrder) -> Self {
        Self {
            name: name.clone(),
            byte_order: byte_order,
            fields: Vec::new(),
        }
    }

    /// Generates the debug_read_data method.
    pub fn generate_debug_read_data(&self) -> TokenStream {
        let mut data_offset: usize = 0;
        let mut debug_read_fields = quote!();
        let mut fields_iterator: Iter<StructureLayoutField> = self.fields.iter();

        while let Some(mut structure_layout_field) = fields_iterator.next() {
            let byte_order: ByteOrder = structure_layout_field.get_byte_order(&self.byte_order);
            let field_name: Ident = format_ident!("{}", structure_layout_field.name);
            let field_type: TokenStream = structure_layout_field.get_type_token_stream();

            let mut value_data_size = match structure_layout_field.get_byte_size() {
                None => panic!(
                    "Unable to determine byte size of field: {}",
                    structure_layout_field.name
                ),
                Some(byte_size) => byte_size,
            };
            let field_format_string: String = match structure_layout_field.format {
                Format::Hexadecimal => format!("0x{{:0{}x}}", value_data_size * 2),
                _ => format!("{{}}"),
            };
            let data_offset_literal: Literal = Literal::usize_unsuffixed(data_offset);
            let end_offset_literal: Literal = Literal::usize_unsuffixed(
                data_offset + (structure_layout_field.number_of_elements * value_data_size),
            );
            if structure_layout_field.data_type == DataType::BitField16
                ||  structure_layout_field.data_type == DataType::BitField32
                || structure_layout_field.data_type == DataType::BitField64
            {
                let bit_field_data_type: DataType = structure_layout_field.data_type.clone();
                let number_of_bits: usize = value_data_size * 8;

                let packed_field_name: Ident = format_ident!("value_{}bit", number_of_bits);
                let quote_data_offset = quote!(#data_offset_literal);
                let quote_from_bytes: TokenStream = structure_layout_field
                    .get_from_bytes_token_stream(&self.byte_order, quote_data_offset);
                let quote_modifier: TokenStream = structure_layout_field.get_modifier_token_stream();

                debug_read_fields.extend(quote! {
                    let #packed_field_name: #field_type = #quote_from_bytes #quote_modifier;

                });
                let mut bit_offset: usize = 0;

                loop {
                    let field_name: Ident = format_ident!("{}", structure_layout_field.name);
                    let field_type = structure_layout_field.get_type_token_stream();

                    let bit_mask: usize = (1 << structure_layout_field.number_of_elements) - 1;
                    let bit_mask_literal = Literal::usize_unsuffixed(bit_mask);
                    let quote_from_packed_value = if bit_offset == 0 {
                        quote!(#packed_field_name & #bit_mask_literal)
                    } else {
                        let bit_offset_literal: Literal = Literal::usize_unsuffixed(bit_offset);
                        quote!((#packed_field_name >> #bit_offset_literal) & #bit_mask_literal)
                    };
                    let number_of_nibbles: usize =
                        structure_layout_field.number_of_elements.div_ceil(4);
                    let field_format_string: String = match structure_layout_field.format {
                        Format::Hexadecimal => format!("0x{{:0{}x}}", number_of_nibbles),
                        _ => format!("{{}}"),
                    };
                    let format_string: String = format!(
                        "    {}: {},\n",
                        structure_layout_field.name, field_format_string
                    );
                    debug_read_fields.extend(quote! {
                        let #field_name: #field_type = #quote_from_packed_value;
                        string_parts.push(format!(#format_string, #field_name));

                    });
                    bit_offset += structure_layout_field.number_of_elements;

                    if bit_offset >= number_of_bits {
                        break;
                    }
                    structure_layout_field = match fields_iterator.next() {
                        Some(structure_layout_field) => {
                            if structure_layout_field.data_type != bit_field_data_type {
                                panic!(
                                    "Unsupported data type of field: {} expected BitField{}",
                                    structure_layout_field.name, number_of_bits
                                );
                            }
                            structure_layout_field
                        }
                        None => break,
                    };
                }
                if bit_offset < number_of_bits {
                    panic!(
                        "BitField{} definition missing for remaining {} bits after field: {}",
                        number_of_bits, number_of_bits - bit_offset, structure_layout_field.name
                    );
                }
            } else if structure_layout_field.data_type == DataType::ByteString
                || structure_layout_field.data_type == DataType::Ucs2String
                || structure_layout_field.data_type == DataType::Utf16String
            {
                if !structure_layout_field.modifier.is_empty() {
                    panic!("Unsupported modifier for data type")
                }
                let format_string: String = format!(
                    "    {}: \"{}\",\n",
                    structure_layout_field.name, field_format_string
                );
                let quote_from_bytes: TokenStream = match &structure_layout_field.data_type {
                    DataType::ByteString => {
                        quote!(from_bytes(&data[#data_offset_literal..#end_offset_literal]))
                    }
                    DataType::Ucs2String | DataType::Utf16String => match byte_order {
                        ByteOrder::BigEndian => {
                            quote!(from_be_bytes(&data[#data_offset_literal..#end_offset_literal]))
                        }
                        ByteOrder::LittleEndian => {
                            quote!(from_le_bytes(&data[#data_offset_literal..#end_offset_literal]))
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                };
                debug_read_fields.extend(quote! {
                    let #field_name: #field_type = #field_type::#quote_from_bytes;
                    string_parts.push(format!(#format_string, #field_name.to_string()));

                });
                value_data_size *= structure_layout_field.number_of_elements;
            } else if structure_layout_field.number_of_elements == 1 {
                match &structure_layout_field.data_type {
                    DataType::Struct { .. } => {
                        let format_string: String =
                            format!("    {}: ", structure_layout_field.name);

                        debug_read_fields.extend(quote! {
                            string_parts.push(format!(#format_string));
                            for line in #field_type::debug_read_data(&data[#data_offset_literal..#end_offset_literal]).lines() {
                                string_parts.push(format!("    {}\n", line));
                            }

                        });
                    }
                    _ => {
                        let quote_data_offset = quote!(#data_offset_literal);
                        let quote_from_bytes: TokenStream = structure_layout_field
                            .get_from_bytes_token_stream(&byte_order, quote_data_offset);
                        let quote_modifier: TokenStream = structure_layout_field.get_modifier_token_stream();

                        let format_string: String = format!(
                            "    {}: {},\n",
                            structure_layout_field.name, field_format_string
                        );
                        let quote_format = match structure_layout_field.format {
                            Format::Character => {
                                quote!(format!(#format_string, #field_name as char))
                            }
                            _ => quote!(format!(#format_string, #field_name)),
                        };
                        debug_read_fields.extend(quote! {
                            let #field_name: #field_type = #quote_from_bytes #quote_modifier;
                            string_parts.push(#quote_format);

                        });
                    }
                };
            } else {
                if !structure_layout_field.modifier.is_empty() {
                    panic!("Unsupported modifier for data type")
                }
                // if structure_layout_field.format == Format::NotSet
                // TODO: add support to format vector of character into string.

                let value_data_size_literal: Literal = Literal::usize_unsuffixed(value_data_size);

                let quote_read_value = match &structure_layout_field.data_type {
                    DataType::Struct { .. } => {
                        // TODO: add , after struct closing } ?
                        let format_string: String =
                            format!("    {}: [\n", structure_layout_field.name);
                        quote!(
                            string_parts.push(format!(#format_string));
                            for data_offset in (#data_offset_literal..#end_offset_literal).step_by(#value_data_size_literal) {
                                for line in #field_type::debug_read_data(&data[data_offset..data_offset + #value_data_size_literal]).lines() {
                                    string_parts.push(format!("        {}\n", line));
                                }
                            }
                            string_parts.push(format!("    ],\n"));
                        )
                    }
                    _ => {
                        let quote_data_offset = quote!(data_offset);
                        let quote_from_bytes: TokenStream = structure_layout_field
                            .get_from_bytes_token_stream(&byte_order, quote_data_offset);
                        let quote_format = match structure_layout_field.format {
                            Format::Character => {
                                quote!(format!(#field_format_string, #field_name as char))
                            }
                            _ => quote!(format!(#field_format_string, #field_name)),
                        };
                        let format_string: String =
                            format!("    {}: {{}},\n", structure_layout_field.name);
                        quote!(
                            let mut array_parts: Vec<String> = Vec::new();
                            for data_offset in (#data_offset_literal..#end_offset_literal).step_by(#value_data_size_literal) {
                                let #field_name: #field_type = #quote_from_bytes;
                                array_parts.push(#quote_format);
                            }
                            string_parts.push(format!(#format_string, crate::formatters::debug_format_array(&array_parts)));
                        )
                    }
                };
                debug_read_fields.extend(quote_read_value);

                value_data_size *= structure_layout_field.number_of_elements;
            }
            data_offset += value_data_size;
        }
        let name_format_string: String = format!("{} {{{{\n", self.name);

        quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!(#name_format_string));

                #debug_read_fields
                string_parts.push(format!("}}\n\n"));

                string_parts.join("")
            }
        }
    }

    /// Generates the read_data_at_position method.
    pub fn generate_read_at_position(&self) -> TokenStream {
        let mut data_size: usize = 0;

        let struct_name: Ident = format_ident!("{}", self.name);
        for structure_layout_field in self.fields.iter() {
            let value_data_size = match structure_layout_field.get_byte_size() {
                None => panic!(
                    "Unable to determine byte size of field: {}",
                    structure_layout_field.name
                ),
                Some(byte_size) => byte_size,
            };
            data_size += value_data_size * structure_layout_field.number_of_elements;
        }
        let data_size_literal: Literal = Literal::usize_unsuffixed(data_size);

        let format_string: String = format!(
            "{} data of size: {{}} at offset: {{}} (0x{{:08x}})\n",
            self.name
        );
        quote! {
            pub(super) fn read_at_position(
                &mut self,
                data_stream: &crate::vfs::VfsDataStreamReference,
                position: std::io::SeekFrom,
            ) -> std::io::Result<()> {
                let mut data: Vec<u8> = vec![0; #data_size_literal];

                let offset: u64 = match data_stream.with_write_lock() {
                    Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                };
                let mediator = crate::mediator::Mediator::current();
                if mediator.debug_output {
                    mediator.debug_print(format!(
                        #format_string,
                        data.len(),
                        offset,
                        offset));
                    mediator.debug_print_data(&data, true);
                    mediator.debug_print(#struct_name::debug_read_data(&data));
                }
                self.read_data(&data)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_debug_read_data() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        structure_layout.fields.push(StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger8Bit,
            ByteOrder::NotSet,
            1,
            &"".to_string(),
            Format::NotSet,
        ));
        structure_layout.fields.push(StructureLayoutField::new(
            &"field2".to_string(),
            DataType::UnsignedInteger16Bit,
            ByteOrder::BigEndian,
            1,
            &"- 7".to_string(),
            Format::NotSet,
        ));
        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: u8 = data[0];
                string_parts.push(format!("    field1: {},\n", field1));

                let field2: u16 = crate::bytes_to_u16_be!(data, 1) - 7;
                string_parts.push(format!("    field2: {},\n", field2));

                string_parts.push(format!("}}\n\n"));

                string_parts.join("")
            }
        };
        let test_token_stream = structure_layout.generate_debug_read_data();
        assert_eq!(
            test_token_stream.to_string(),
            expected_token_stream.to_string()
        );
    }

    #[test]
    fn test_generate_debug_read_data_with_array_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        structure_layout.fields.push(StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger64Bit,
            ByteOrder::BigEndian,
            1,
            &"".to_string(),
            Format::NotSet,
        ));
        structure_layout.fields.push(StructureLayoutField::new(
            &"field2".to_string(),
            DataType::UnsignedInteger8Bit,
            ByteOrder::NotSet,
            64,
            &"".to_string(),
            Format::NotSet,
        ));
        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: u64 = crate::bytes_to_u64_be!(data, 0);
                string_parts.push(format!("    field1: {},\n", field1));

                let mut array_parts: Vec<String> = Vec::new();
                for data_offset in (8..72).step_by(1) {
                    let field2: u8 = data[data_offset];
                    array_parts.push(format!("{}", field2));
                }
                string_parts.push(format!("    field2: {},\n", crate::formatters::debug_format_array(&array_parts)));

                string_parts.push(format!("}}\n\n"));

                string_parts.join("")
            }
        };
        let test_token_stream = structure_layout.generate_debug_read_data();
        assert_eq!(
            test_token_stream.to_string(),
            expected_token_stream.to_string()
        );
    }

    #[test]
    fn test_generate_debug_read_data_with_bit_fields() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::LittleEndian);

        structure_layout.fields.push(StructureLayoutField::new(
            &"field1".to_string(),
            DataType::BitField32,
            ByteOrder::NotSet,
            9,
            &"".to_string(),
            Format::NotSet,
        ));
        structure_layout.fields.push(StructureLayoutField::new(
            &"field2".to_string(),
            DataType::BitField32,
            ByteOrder::NotSet,
            23,
            &"".to_string(),
            Format::NotSet,
        ));
        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let value_32bit: u32 = crate::bytes_to_u32_le!(data, 0);

                let field1: u32 = value_32bit & 511;
                string_parts.push(format!("    field1: {},\n", field1));

                let field2: u32 = (value_32bit >> 9) & 8388607;
                string_parts.push(format!("    field2: {},\n", field2));

                string_parts.push(format!("}}\n\n"));

                string_parts.join("")
            }
        };
        let test_token_stream = structure_layout.generate_debug_read_data();

        // let test_file = syn::parse_file(&test_token_stream.to_string()).unwrap();
        // let expected_file = syn::parse_file(&expected_token_stream.to_string()).unwrap();

        // assert_eq!(
        //     prettyplease::unparse(&test_file),
        //     prettyplease::unparse(&expected_file),
        // );
        assert_eq!(
            test_token_stream.to_string(),
            expected_token_stream.to_string()
        );
    }

    #[test]
    fn test_generate_debug_read_data_with_struct_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        structure_layout.fields.push(StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger32Bit,
            ByteOrder::LittleEndian,
            1,
            &"".to_string(),
            Format::NotSet,
        ));
        structure_layout.fields.push(StructureLayoutField::new(
            &"field2".to_string(),
            DataType::Struct {
                name: "MyStruct".to_string(),
                size: 7,
            },
            ByteOrder::NotSet,
            1,
            &"".to_string(),
            Format::NotSet,
        ));
        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: u32 = crate::bytes_to_u32_le!(data, 0);
                string_parts.push(format!("    field1: {},\n", field1));

                string_parts.push(format!("    field2: "));
                for line in MyStruct::debug_read_data(&data[4..11]).lines() {
                    string_parts.push(format!("    {}\n", line));
                }
                string_parts.push(format!("}}\n\n"));

                string_parts.join("")
            }
        };
        let test_token_stream = structure_layout.generate_debug_read_data();
        assert_eq!(
            test_token_stream.to_string(),
            expected_token_stream.to_string()
        );
    }

    #[test]
    fn test_generate_debug_read_data_with_struct_array_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        structure_layout.fields.push(StructureLayoutField::new(
            &"field1".to_string(),
            DataType::SignedInteger32Bit,
            ByteOrder::LittleEndian,
            1,
            &"".to_string(),
            Format::NotSet,
        ));
        structure_layout.fields.push(StructureLayoutField::new(
            &"field2".to_string(),
            DataType::Struct {
                name: "MyStruct".to_string(),
                size: 5,
            },
            ByteOrder::NotSet,
            10,
            &"".to_string(),
            Format::NotSet,
        ));
        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: i32 = crate::bytes_to_i32_le!(data, 0);
                string_parts.push(format!("    field1: {},\n", field1));

                string_parts.push(format!("    field2: [\n"));
                for data_offset in (4..54).step_by(5) {
                    for line in MyStruct::debug_read_data(&data[data_offset..data_offset + 5]).lines() {
                        string_parts.push(format!("        {}\n", line));
                    }
                }
                string_parts.push(format!("    ],\n"));

                string_parts.push(format!("}}\n\n"));

                string_parts.join("")
            }
        };
        let test_token_stream = structure_layout.generate_debug_read_data();

        assert_eq!(
            test_token_stream.to_string(),
            expected_token_stream.to_string()
        );
    }

    #[test]
    fn test_read_at_position() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        structure_layout.fields.push(StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger8Bit,
            ByteOrder::NotSet,
            16,
            &"".to_string(),
            Format::NotSet,
        ));
        let expected_token_stream = quote! {
            pub(super) fn read_at_position(
                &mut self,
                data_stream: &crate::vfs::VfsDataStreamReference,
                position: std::io::SeekFrom,
            ) -> std::io::Result<()> {
                let mut data: Vec<u8> = vec![0; 16];

                let offset: u64 = match data_stream.with_write_lock() {
                    Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                };
                let mediator = crate::mediator::Mediator::current();
                if mediator.debug_output {
                    mediator.debug_print(format!(
                        "TestStruct data of size: {} at offset: {} (0x{:08x})\n",
                        data.len(),
                        offset,
                        offset));
                    mediator.debug_print_data(&data, true);
                    mediator.debug_print(TestStruct::debug_read_data(&data));
                }
                self.read_data(&data)
            }
        };
        let test_token_stream = structure_layout.generate_read_at_position();
        assert_eq!(
            test_token_stream.to_string(),
            expected_token_stream.to_string()
        );
    }
}
