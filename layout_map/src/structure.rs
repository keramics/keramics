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

use std::str::FromStr;

use proc_macro2::{Ident, Literal, TokenStream};

use quote::{format_ident, quote};

use crate::enums::{ByteOrder, DataType, Format};

/// Structure layout bitfield.
pub(crate) struct StructureLayoutBitField {
    /// Name.
    pub name: String,

    /// Number of bits.
    pub number_of_bits: usize,

    /// Modifier.
    pub modifier: String,

    /// Format.
    pub format: Format,
}

impl StructureLayoutBitField {
    /// Creates a new bitfield.
    pub fn new(name: &String, number_of_bits: usize, modifier: &String, format: Format) -> Self {
        Self {
            name: name.clone(),
            number_of_bits: number_of_bits,
            modifier: modifier.clone(),
            format: format,
        }
    }

    /// Retrieves the format string.
    pub fn get_format_string(&self) -> String {
        match self.format {
            Format::Hexadecimal => {
                let number_of_nibbles: usize = self.number_of_bits.div_ceil(4);
                format!("0x{{:0{}x}}", number_of_nibbles)
            }
            _ => format!("{{}}"),
        }
    }

    /// Retrieves a token stream of the modifier.
    pub fn get_modifier_token_stream(&self) -> TokenStream {
        match TokenStream::from_str(&self.modifier) {
            Ok(token_stream) => token_stream,
            Err(error) => panic!(
                "Unable to parse modifier: \"{}\" with error: {}",
                self.modifier, error
            ),
        }
    }
}

/// Structure layout bitfields group.
pub(crate) struct StructureLayoutBitFieldsGroup {
    /// Data type.
    pub data_type: DataType,

    /// Byte order.
    pub byte_order: ByteOrder,

    /// Bitfields.
    pub bitfields: Vec<StructureLayoutBitField>,
}

impl StructureLayoutBitFieldsGroup {
    /// Creates a new group.
    pub fn new(data_type: DataType, byte_order: ByteOrder) -> Self {
        Self {
            data_type: data_type,
            byte_order: byte_order,
            bitfields: Vec::new(),
        }
    }

    /// Retrieves the byte order.
    pub fn get_byte_order(&self, parent_byte_order: &ByteOrder) -> ByteOrder {
        if self.byte_order != ByteOrder::NotSet {
            return self.byte_order.clone();
        }
        if parent_byte_order == &ByteOrder::NotSet {
            panic!("Byte order missing in bitfields");
        }
        parent_byte_order.clone()
    }

    /// Retrieves the byte size.
    pub fn get_byte_size(&self) -> Option<usize> {
        match &self.data_type {
            DataType::BitField8 => Some(1),
            DataType::BitField16 => Some(2),
            DataType::BitField32 => Some(4),
            DataType::BitField64 => Some(8),
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
            DataType::BitField8 => quote!(data[#data_offset]),
            DataType::BitField16 => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u16_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u16_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::BitField32 => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u32_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u32_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::BitField64 => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u64_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u64_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            _ => panic!("Unsupported data type"),
        }
    }

    /// Retrieves a token stream of the type.
    pub fn get_type_token_stream(&self) -> TokenStream {
        match &self.data_type {
            DataType::BitField8 => quote!(u8),
            DataType::BitField16 => quote!(u16),
            DataType::BitField32 => quote!(u32),
            DataType::BitField64 => quote!(u64),
            _ => panic!("Unsupported data type"),
        }
    }
}

/// Structure layout field.
pub(crate) struct StructureLayoutField {
    /// Name.
    pub name: String,

    /// Data type.
    pub data_type: DataType,

    /// Byte order.
    pub byte_order: ByteOrder,

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
        modifier: &String,
        format: Format,
    ) -> Self {
        Self {
            name: name.clone(),
            data_type: data_type,
            byte_order: byte_order,
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
            DataType::ByteString => Some(1),
            DataType::Filetime => Some(8),
            DataType::PosixTime32 => Some(4),
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

    /// Retrieves a token stream to debug read the field.
    pub fn get_debug_read(
        &self,
        byte_order: &ByteOrder,
        value_data_size: usize,
        data_offset: usize,
    ) -> TokenStream {
        let data_end_offset: usize = data_offset + value_data_size;

        let field_type: TokenStream = self.get_type_token_stream();
        let data_offset_literal: Literal = Literal::usize_unsuffixed(data_offset);
        let data_end_offset_literal: Literal = Literal::usize_unsuffixed(data_end_offset);

        let field_format_string: String = match self.format {
            Format::Hexadecimal => format!("0x{{:0{}x}}", value_data_size * 2),
            _ => format!("{{}}"),
        };
        match &self.data_type {
            DataType::Struct { .. } => {
                let format_string: String = format!("    {}: ", self.name);

                quote! {
                    string_parts.push(format!(#format_string));
                    for line in #field_type::debug_read_data(&data[#data_offset_literal..#data_end_offset_literal]).lines() {
                        string_parts.push(format!("    {}\n", line));
                    }

                }
            }
            _ => {
                let field_name: Ident = format_ident!("{}", self.name);

                let quote_data_offset = quote!(#data_offset_literal);
                let quote_from_bytes: TokenStream =
                    self.get_from_bytes_token_stream(&byte_order, quote_data_offset);
                let field_modifier: TokenStream = self.get_modifier_token_stream();

                let format_string: String =
                    format!("    {}: {},\n", self.name, field_format_string);

                let quote_format = match self.format {
                    Format::Character => quote!(format!(#format_string, #field_name as char)),
                    _ => quote!(format!(#format_string, #field_name)),
                };
                quote! {
                    let #field_name: #field_type = #quote_from_bytes #field_modifier;
                    string_parts.push(#quote_format);

                }
            }
        }
    }

    /// Retrieves a token stream to debug read the field as a sequence.
    pub fn get_debug_read_sequence(
        &self,
        byte_order: &ByteOrder,
        data_offset: usize,
        element_data_size: usize,
        number_of_elements: usize,
    ) -> TokenStream {
        // if structure_layout_sequence.element.format == Format::NotSet
        // TODO: add support to format vector of character into string.

        let field_name: Ident = format_ident!("{}", self.name);
        let field_type: TokenStream = self.get_type_token_stream();

        let data_end_offset: usize = data_offset + (number_of_elements * element_data_size);

        let data_offset_literal: Literal = Literal::usize_unsuffixed(data_offset);
        let data_end_offset_literal: Literal = Literal::usize_unsuffixed(data_end_offset);
        let element_data_size_literal: Literal = Literal::usize_unsuffixed(element_data_size);

        match &self.data_type {
            DataType::ByteString => {
                let format_string: String = format!("    {}: \"{{}}\",\n", self.name);

                quote! {
                    let #field_name: #field_type = #field_type::from_bytes(&data[#data_offset_literal..#data_end_offset_literal]);
                    string_parts.push(format!(#format_string, #field_name.to_string()));

                }
            }
            DataType::Ucs2String | DataType::Utf16String => {
                let format_string: String = format!("    {}: \"{{}}\",\n", self.name);
                let quote_from_bytes: TokenStream = match byte_order {
                    ByteOrder::BigEndian => {
                        quote!(from_be_bytes(&data[#data_offset_literal..#data_end_offset_literal]))
                    }
                    ByteOrder::LittleEndian => {
                        quote!(from_le_bytes(&data[#data_offset_literal..#data_end_offset_literal]))
                    }
                    _ => panic!("Unsupported byte order"),
                };
                quote! {
                    let #field_name: #field_type = #field_type::#quote_from_bytes;
                    string_parts.push(format!(#format_string, #field_name.to_string()));

                }
            }
            DataType::Struct { .. } => {
                // TODO: add , after struct closing } ?
                let format_string: String = format!("    {}: [\n", self.name);
                quote!(
                    string_parts.push(format!(#format_string));
                    for data_offset in (#data_offset_literal..#data_end_offset_literal).step_by(#element_data_size_literal) {
                        for line in #field_type::debug_read_data(&data[data_offset..data_offset + #element_data_size_literal]).lines() {
                            string_parts.push(format!("        {}\n", line));
                        }
                    }
                    string_parts.push(format!("    ],\n"));
                )
            }
            _ => {
                let field_format_string: String = match self.format {
                    Format::Hexadecimal => format!("0x{{:0{}x}}", element_data_size * 2),
                    _ => format!("{{}}"),
                };
                let quote_data_offset = quote!(data_offset);
                let quote_from_bytes: TokenStream =
                    self.get_from_bytes_token_stream(&byte_order, quote_data_offset);
                let quote_format = match self.format {
                    Format::Character => {
                        quote!(format!(#field_format_string, #field_name as char))
                    }
                    _ => quote!(format!(#field_format_string, #field_name)),
                };
                let format_string: String = format!("    {}: {{}},\n", self.name);
                quote!(
                    let mut array_parts: Vec<String> = Vec::new();
                    for data_offset in (#data_offset_literal..#data_end_offset_literal).step_by(#element_data_size_literal) {
                        let #field_name: #field_type = #quote_from_bytes;
                        array_parts.push(#quote_format);
                    }
                    string_parts.push(format!(#format_string, crate::formatters::debug_format_array(&array_parts)));
                )
            }
        }
    }

    /// Retrieves a token stream to convert the field from bytes.
    pub fn get_from_bytes_token_stream(
        &self,
        byte_order: &ByteOrder,
        data_offset: TokenStream,
    ) -> TokenStream {
        match &self.data_type {
            DataType::Filetime => {
                quote!(crate::datetime::Filetime::from_bytes(&data[#data_offset..#data_offset + 8]))
            }
            DataType::PosixTime32 => match byte_order {
                &ByteOrder::BigEndian => {
                    quote!(crate::datetime::PosixTime32::from_be_bytes(&data[#data_offset..#data_offset + 4]))
                }
                &ByteOrder::LittleEndian => {
                    quote!(crate::datetime::PosixTime32::from_le_bytes(&data[#data_offset..#data_offset + 4]))
                }
                _ => panic!("Unsupported byte order"),
            },
            DataType::SignedInteger8Bit | DataType::UnsignedInteger8Bit => {
                quote!(data[#data_offset])
            }
            DataType::SignedInteger16Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_i16_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_i16_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::SignedInteger32Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_i32_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_i32_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::SignedInteger64Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_i64_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_i64_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::UnsignedInteger16Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u16_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u16_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::UnsignedInteger32Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u32_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u32_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::UnsignedInteger64Bit => match byte_order {
                &ByteOrder::BigEndian => quote!(crate::bytes_to_u64_be!(data, #data_offset)),
                &ByteOrder::LittleEndian => quote!(crate::bytes_to_u64_le!(data, #data_offset)),
                _ => panic!("Unsupported byte order"),
            },
            DataType::Uuid => match byte_order {
                &ByteOrder::BigEndian => {
                    quote!(crate::types::Uuid::from_be_bytes(&data[#data_offset..#data_offset + 16]))
                }
                &ByteOrder::LittleEndian => {
                    quote!(crate::types::Uuid::from_le_bytes(&data[#data_offset..#data_offset + 16]))
                }
                _ => panic!("Unsupported byte order"),
            },
            _ => panic!("Unsupported data type"),
        }
    }

    /// Retrieves a token stream of the modifier.
    pub fn get_modifier_token_stream(&self) -> TokenStream {
        match TokenStream::from_str(&self.modifier) {
            Ok(token_stream) => token_stream,
            Err(error) => panic!(
                "Unable to parse modifier: \"{}\" with error: {}",
                self.modifier, error
            ),
        }
    }

    /// Retrieves a token stream of the type.
    pub fn get_type_token_stream(&self) -> TokenStream {
        match &self.data_type {
            DataType::ByteString => quote!(crate::types::ByteString),
            DataType::Filetime => quote!(crate::datetime::Filetime),
            DataType::PosixTime32 => quote!(crate::datetime::PosixTime32),
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
            _ => panic!("Unsupported data type"),
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

/// Structure layout group.
pub(crate) struct StructureLayoutGroup {
    /// Condition.
    pub condition: String,

    /// Fields.
    pub fields: Vec<StructureLayoutField>,
}

impl StructureLayoutGroup {
    /// Creates a new group.
    pub fn new(condition: &String) -> Self {
        Self {
            condition: condition.clone(),
            fields: Vec::new(),
        }
    }

    /// Retrieves a token stream of the condition.
    pub fn get_condition_token_stream(&self) -> TokenStream {
        match TokenStream::from_str(&self.condition) {
            Ok(token_stream) => token_stream,
            Err(error) => panic!(
                "Unable to parse condition: \"{}\" with error: {}",
                self.condition, error
            ),
        }
    }

    /// Retrieves the data size.
    pub fn get_data_size(&self) -> usize {
        let mut data_size: usize = 0;

        for field in self.fields.iter() {
            let byte_size: usize = match field.get_byte_size() {
                Some(byte_size) => byte_size,
                None => panic!("Unable to determine byte size of field: {}", field.name),
            };
            data_size += byte_size;
        }
        data_size
    }
}

/// Structure layout sequence.
pub(crate) struct StructureLayoutSequence {
    /// Element.
    pub element: StructureLayoutField,

    /// Number of elements.
    pub number_of_elements: usize,
}

impl StructureLayoutSequence {
    /// Creates a new sequence.
    pub fn new(element: StructureLayoutField, number_of_elements: usize) -> Self {
        Self {
            element: element,
            number_of_elements: number_of_elements,
        }
    }

    /// Retrieves the data size.
    pub fn get_data_size(&self) -> usize {
        let byte_size: usize = match self.element.get_byte_size() {
            Some(byte_size) => byte_size,
            None => panic!(
                "Unable to determine byte size of field: {}",
                self.element.name
            ),
        };
        byte_size * self.number_of_elements
    }
}

/// Structure layout member.
pub enum StructureLayoutMember {
    /// Bitfields group.
    BitFields(StructureLayoutBitFieldsGroup),

    /// Field.
    Field(StructureLayoutField),

    /// Group.
    Group(StructureLayoutGroup),

    /// Sequence.
    Sequence(StructureLayoutSequence),
}

/// Structure layout.
pub(crate) struct StructureLayout {
    /// Name.
    pub name: String,

    /// Byte order.
    pub byte_order: ByteOrder,

    /// Members.
    pub members: Vec<StructureLayoutMember>,
}

impl StructureLayout {
    /// Creates a new layout.
    pub fn new(name: &String, byte_order: ByteOrder) -> Self {
        Self {
            name: name.clone(),
            byte_order: byte_order,
            members: Vec::new(),
        }
    }

    /// Generates part of the debug_read_data method for a bitfields group.
    fn generate_debug_read_bitfields(
        &self,
        data_offset: usize,
        bitfields_group: &StructureLayoutBitFieldsGroup,
    ) -> TokenStream {
        let mut debug_read_bitfields = quote!();

        let byte_order: ByteOrder = bitfields_group.get_byte_order(&self.byte_order);
        let field_type: TokenStream = bitfields_group.get_type_token_stream();

        let value_data_size = match bitfields_group.get_byte_size() {
            Some(byte_size) => byte_size,
            None => panic!("Unable to determine byte size of bitfields",),
        };
        let number_of_bits: usize = value_data_size * 8;

        let data_offset_literal: Literal = Literal::usize_unsuffixed(data_offset);

        let packed_field_name: Ident = format_ident!("value_{}bit", number_of_bits);
        let quote_data_offset = quote!(#data_offset_literal);
        let quote_from_bytes: TokenStream =
            bitfields_group.get_from_bytes_token_stream(&byte_order, quote_data_offset);

        debug_read_bitfields.extend(quote! {
            let #packed_field_name: #field_type = #quote_from_bytes;

        });
        let mut bit_offset: usize = 0;

        for bitfield in bitfields_group.bitfields.iter() {
            let field_name: Ident = format_ident!("{}", bitfield.name);

            let bit_mask: usize = (1 << bitfield.number_of_bits) - 1;
            let bit_mask_literal = Literal::usize_unsuffixed(bit_mask);
            let quote_from_packed_value = if bit_offset == 0 {
                quote!(#packed_field_name & #bit_mask_literal)
            } else {
                let bit_offset_literal: Literal = Literal::usize_unsuffixed(bit_offset);
                quote!((#packed_field_name >> #bit_offset_literal) & #bit_mask_literal)
            };
            let field_modifier: TokenStream = bitfield.get_modifier_token_stream();

            let field_format_string: String = bitfield.get_format_string();
            let format_string: String =
                format!("    {}: {},\n", bitfield.name, field_format_string);
            debug_read_bitfields.extend(quote! {
                let #field_name: #field_type = #quote_from_packed_value #field_modifier;
                string_parts.push(format!(#format_string, #field_name));

            });
            bit_offset += bitfield.number_of_bits;
        }
        debug_read_bitfields
    }

    /// Generates part of the debug_read_data method for a group.
    fn generate_debug_read_group(
        &self,
        mut data_offset: usize,
        group: &StructureLayoutGroup,
    ) -> TokenStream {
        let mut debug_read_fields = quote!();

        for field in group.fields.iter() {
            let byte_order: ByteOrder = field.get_byte_order(&self.byte_order);

            let value_data_size = match field.get_byte_size() {
                Some(byte_size) => byte_size,
                None => panic!("Unable to determine byte size of field: {}", field.name),
            };
            let debug_read_field: TokenStream =
                field.get_debug_read(&byte_order, value_data_size, data_offset);
            debug_read_fields.extend(debug_read_field);

            data_offset += value_data_size;
        }
        let group_condition: TokenStream = group.get_condition_token_stream();
        quote! {
            if #group_condition {
                #debug_read_fields
            }
        }
    }

    /// Generates the debug_read_data method.
    pub fn generate_debug_read_data(&self) -> TokenStream {
        let mut data_offset: usize = 0;
        let mut debug_read_fields = quote!();

        for structure_layout_member in self.members.iter() {
            match structure_layout_member {
                StructureLayoutMember::BitFields(structure_layout_bitfields) => {
                    let value_data_size = match structure_layout_bitfields.get_byte_size() {
                        Some(byte_size) => byte_size,
                        None => panic!("Unable to determine byte size of bitfields",),
                    };
                    let debug_read_bitfields: TokenStream = self
                        .generate_debug_read_bitfields(data_offset, &structure_layout_bitfields);
                    debug_read_fields.extend(debug_read_bitfields);

                    data_offset += value_data_size;
                }
                StructureLayoutMember::Field(structure_layout_field) => {
                    let byte_order: ByteOrder =
                        structure_layout_field.get_byte_order(&self.byte_order);

                    let value_data_size = match structure_layout_field.get_byte_size() {
                        Some(byte_size) => byte_size,
                        None => panic!(
                            "Unable to determine byte size of field: {}",
                            structure_layout_field.name
                        ),
                    };
                    let debug_read_field: TokenStream = structure_layout_field.get_debug_read(
                        &byte_order,
                        value_data_size,
                        data_offset,
                    );
                    debug_read_fields.extend(debug_read_field);

                    data_offset += value_data_size;
                }
                StructureLayoutMember::Group(structure_layout_group) => {
                    let debug_read_group: TokenStream =
                        self.generate_debug_read_group(data_offset, &structure_layout_group);
                    debug_read_fields.extend(debug_read_group);

                    let group_data_size = structure_layout_group.get_data_size();
                    data_offset += group_data_size;
                }
                StructureLayoutMember::Sequence(structure_layout_sequence) => {
                    let byte_order: ByteOrder = structure_layout_sequence
                        .element
                        .get_byte_order(&self.byte_order);

                    let element_data_size = match structure_layout_sequence.element.get_byte_size()
                    {
                        Some(byte_size) => byte_size,
                        None => panic!(
                            "Unable to determine byte size of field: {}",
                            structure_layout_sequence.element.name
                        ),
                    };
                    let debug_read_field: TokenStream =
                        structure_layout_sequence.element.get_debug_read_sequence(
                            &byte_order,
                            data_offset,
                            element_data_size,
                            structure_layout_sequence.number_of_elements,
                        );
                    debug_read_fields.extend(debug_read_field);

                    data_offset += element_data_size * structure_layout_sequence.number_of_elements;
                }
            };
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
        let struct_name: Ident = format_ident!("{}", self.name);

        let data_size: usize = self.get_data_size();
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

    /// Retrieves the data size.
    fn get_data_size(&self) -> usize {
        let mut data_size: usize = 0;

        for structure_layout_member in self.members.iter() {
            let value_data_size: usize = match structure_layout_member {
                StructureLayoutMember::BitFields(structure_layout_bitfields) => {
                    match structure_layout_bitfields.get_byte_size() {
                        Some(byte_size) => byte_size,
                        None => panic!("Unable to determine byte size of bitfields",),
                    }
                }
                StructureLayoutMember::Field(structure_layout_field) => {
                    match structure_layout_field.get_byte_size() {
                        Some(byte_size) => byte_size,
                        None => panic!(
                            "Unable to determine byte size of field: {}",
                            structure_layout_field.name
                        ),
                    }
                }
                StructureLayoutMember::Group(structure_layout_group) => {
                    structure_layout_group.get_data_size()
                }
                StructureLayoutMember::Sequence(structure_layout_sequence) => {
                    structure_layout_sequence.get_data_size()
                }
            };
            data_size += value_data_size;
        }
        data_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_debug_read_data() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger8Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field2".to_string(),
            DataType::UnsignedInteger16Bit,
            ByteOrder::LittleEndian,
            &"- 7".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: u8 = data[0];
                string_parts.push(format!("    field1: {},\n", field1));

                let field2: u16 = crate::bytes_to_u16_le!(data, 1) - 7;
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
    fn test_generate_debug_read_data_with_sequence_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger64Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field2".to_string(),
            DataType::UnsignedInteger8Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        let sequence: StructureLayoutSequence = StructureLayoutSequence::new(field, 64);
        structure_layout
            .members
            .push(StructureLayoutMember::Sequence(sequence));

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
    fn test_generate_debug_read_data_with_string_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger64Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field2".to_string(),
            DataType::ByteString,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        let sequence: StructureLayoutSequence = StructureLayoutSequence::new(field, 32);
        structure_layout
            .members
            .push(StructureLayoutMember::Sequence(sequence));

        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: u64 = crate::bytes_to_u64_be!(data, 0);
                string_parts.push(format!("    field1: {},\n", field1));

                let field2: crate::types::ByteString = crate::types::ByteString::from_bytes(&data[8..40]);
                string_parts.push(format!("    field2: \"{}\",\n", field2.to_string()));

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

        let mut group: StructureLayoutBitFieldsGroup =
            StructureLayoutBitFieldsGroup::new(DataType::BitField32, ByteOrder::NotSet);

        let bitfield: StructureLayoutBitField =
            StructureLayoutBitField::new(&"field1".to_string(), 9, &"".to_string(), Format::NotSet);
        group.bitfields.push(bitfield);

        let bitfield: StructureLayoutBitField = StructureLayoutBitField::new(
            &"field2".to_string(),
            23,
            &"".to_string(),
            Format::NotSet,
        );
        group.bitfields.push(bitfield);

        structure_layout
            .members
            .push(StructureLayoutMember::BitFields(group));

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
    fn test_generate_debug_read_data_with_group() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger64Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let mut group: StructureLayoutGroup =
            StructureLayoutGroup::new(&"data.len() > 8".to_string());

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field2".to_string(),
            DataType::UnsignedInteger32Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        group.fields.push(field);

        structure_layout
            .members
            .push(StructureLayoutMember::Group(group));

        let expected_token_stream = quote! {
            pub fn debug_read_data(data: &[u8]) -> String {
                let mut string_parts: Vec<String> = Vec::new();
                string_parts.push(format!("TestStruct {{\n"));

                let field1: u64 = crate::bytes_to_u64_be!(data, 0);
                string_parts.push(format!("    field1: {},\n", field1));

                if data.len() > 8 {
                    let field2: u32 = crate::bytes_to_u32_be!(data, 8);
                    string_parts.push(format!("    field2: {},\n", field2));
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
    fn test_generate_debug_read_data_with_struct_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger32Bit,
            ByteOrder::LittleEndian,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field2".to_string(),
            DataType::Struct {
                name: "MyStruct".to_string(),
                size: 7,
            },
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

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
    fn test_generate_debug_read_data_with_struct_sequence_field() {
        let mut structure_layout: StructureLayout =
            StructureLayout::new(&"TestStruct".to_string(), ByteOrder::BigEndian);

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::SignedInteger32Bit,
            ByteOrder::LittleEndian,
            &"".to_string(),
            Format::NotSet,
        );
        structure_layout
            .members
            .push(StructureLayoutMember::Field(field));

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field2".to_string(),
            DataType::Struct {
                name: "MyStruct".to_string(),
                size: 5,
            },
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        let sequence: StructureLayoutSequence = StructureLayoutSequence::new(field, 10);
        structure_layout
            .members
            .push(StructureLayoutMember::Sequence(sequence));

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

        let field: StructureLayoutField = StructureLayoutField::new(
            &"field1".to_string(),
            DataType::UnsignedInteger8Bit,
            ByteOrder::NotSet,
            &"".to_string(),
            Format::NotSet,
        );
        let sequence: StructureLayoutSequence = StructureLayoutSequence::new(field, 16);
        structure_layout
            .members
            .push(StructureLayoutMember::Sequence(sequence));

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
