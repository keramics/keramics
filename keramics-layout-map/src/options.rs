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

use syn::parse::{Parse, ParseStream};

use super::enums::{ByteOrder, DataType, Format};

/// Byte order option.
#[derive(Debug, PartialEq)]
pub struct ByteOrderOption {
    /// Value.
    value: ByteOrder,
}

impl ByteOrderOption {
    /// Retrieves the value.
    pub fn value(&self) -> ByteOrder {
        self.value.clone()
    }
}

impl Parse for ByteOrderOption {
    /// Parses the option from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let string_value: String = input.parse::<syn::LitStr>()?.value();

        let value: ByteOrder = match string_value.as_str() {
            "" => ByteOrder::NotSet,
            "be" | "big" | "BigEndian" => ByteOrder::BigEndian,
            "le" | "little" | "LittleEndian" => ByteOrder::LittleEndian,
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    format!("Unsupported byte order: {}", string_value),
                ));
            }
        };
        Ok(Self { value })
    }
}

/// Field data type option.
#[derive(Debug, PartialEq)]
pub struct FieldDataTypeOption {
    /// Value.
    value: DataType,

    /// Number of elements.
    pub number_of_elements: usize,
}

impl FieldDataTypeOption {
    /// Retrieves the value.
    pub fn value(&self) -> DataType {
        self.value.clone()
    }
}

impl Parse for FieldDataTypeOption {
    /// Parses the option from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let string_value: String = input.parse::<syn::LitStr>()?.value();

        let mut data_type_str: &str = string_value.as_str();
        let mut number_of_elements_str: &str = "";
        let mut extended_type_str: &str = "";

        if data_type_str.starts_with("[") && data_type_str.ends_with("]") {
            let string_size: usize = data_type_str.len();

            (data_type_str, number_of_elements_str) =
                data_type_str[1..string_size - 1].rsplit_once(";").unwrap();
            data_type_str = data_type_str.trim();
            number_of_elements_str = number_of_elements_str.trim();
        }
        if data_type_str.ends_with(">") {
            match data_type_str.chars().rev().position(|value| value == '<') {
                Some(value_index) => {
                    let string_size: usize = data_type_str.len();
                    // Note that value_index is relative to end of the string.
                    let value_index: usize = string_size - value_index - 1;

                    extended_type_str = &data_type_str[value_index + 1..string_size - 1];
                    data_type_str = &data_type_str[0..value_index];
                }
                None => {}
            }
        }
        let value: DataType = match data_type_str {
            "BitField8" => DataType::BitField8,
            "BitField16" => DataType::BitField16,
            "BitField32" => DataType::BitField32,
            "BitField64" => DataType::BitField64,
            "ByteString" => DataType::ByteString,
            "FatDate" => DataType::FatDate,
            "FatTimeDate" => DataType::FatTimeDate,
            "FatTimeDate10Ms" => DataType::FatTimeDate10Ms,
            "Filetime" => DataType::Filetime,
            "i8" | "int8" | "SignedInteger8Bit" => DataType::SignedInteger8Bit,
            "i16" | "int16" | "SignedInteger16Bit" => DataType::SignedInteger16Bit,
            "i32" | "int32" | "SignedInteger32Bit" => DataType::SignedInteger32Bit,
            "i64" | "int64" | "SignedInteger64Bit" => DataType::SignedInteger64Bit,
            "HfsTime" => DataType::HfsTime,
            "PosixTime32" => DataType::PosixTime32,
            "Struct" => {
                let (struct_name_str, struct_size_str) = extended_type_str.split_once(";").unwrap();
                extended_type_str = "";

                DataType::Struct {
                    name: struct_name_str.trim().to_string(),
                    size: struct_size_str.trim().parse::<usize>().unwrap(),
                }
            }
            "u8" | "uint8" | "UnsignedInteger8Bit" => DataType::UnsignedInteger8Bit,
            "u16" | "uint16" | "UnsignedInteger16Bit" => DataType::UnsignedInteger16Bit,
            "u32" | "uint32" | "UnsignedInteger32Bit" => DataType::UnsignedInteger32Bit,
            "u64" | "uint64" | "UnsignedInteger64Bit" => DataType::UnsignedInteger64Bit,
            "uuid" | "Uuid" => DataType::Uuid,
            "Ucs2String" => DataType::Ucs2String,
            "Utf16String" => DataType::Utf16String,
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    format!("Unsupported data type: {}", data_type_str),
                ));
            }
        };
        let mut number_of_elements: usize = 1;

        if !extended_type_str.is_empty() {
            number_of_elements_str = extended_type_str;
        }
        if !number_of_elements_str.is_empty() {
            number_of_elements = match number_of_elements_str.parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(syn::Error::new(
                        input.span(),
                        format!(
                            "Unsupported number of elements: {} in data type: {}",
                            number_of_elements_str, data_type_str
                        ),
                    ));
                }
            }
        }
        Ok(Self {
            value,
            number_of_elements,
        })
    }
}

/// Field format option.
#[derive(Debug, PartialEq)]
pub struct FieldFormatOption {
    /// Value.
    value: Format,
}

impl FieldFormatOption {
    /// Retrieves the value.
    pub fn value(&self) -> Format {
        self.value.clone()
    }
}

impl Parse for FieldFormatOption {
    /// Parses the option from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let string_value: String = input.parse::<syn::LitStr>()?.value();

        let value: Format = match string_value.as_str() {
            "" => Format::NotSet,
            "char" | "Character" => Format::Character,
            "hex" | "Hexadecimal" => Format::Hexadecimal,
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    format!("Unsupported format: {}", string_value),
                ));
            }
        };
        Ok(Self { value })
    }
}

/// Field options.
#[derive(Debug, PartialEq)]
pub struct FieldOptions {
    /// Byte order.
    pub byte_order: ByteOrder,

    /// Data type.
    pub data_type: DataType,

    /// Format.
    pub format: Format,

    /// Modifier.
    pub modifier: String,

    /// Name.
    pub name: String,

    /// Number of elements.
    pub number_of_elements: usize,
}

impl FieldOptions {
    /// Creates new options.
    pub fn new() -> Self {
        Self {
            byte_order: ByteOrder::NotSet,
            data_type: DataType::NotSet,
            format: Format::NotSet,
            modifier: String::new(),
            name: String::new(),
            number_of_elements: 0,
        }
    }
}

impl Parse for FieldOptions {
    /// Parses the options from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut options: Self = Self::new();

        while !input.is_empty() {
            if let Ok(ident) = input.parse::<syn::Ident>() {
                let identifier: String = ident.to_string();

                input.parse::<syn::token::Eq>()?;

                match identifier.as_str() {
                    "byte_order" => {
                        options.byte_order = input.parse::<ByteOrderOption>()?.value();
                    }
                    "data_type" => {
                        let data_type_option: FieldDataTypeOption = input.parse()?;
                        options.data_type = data_type_option.value();
                        options.number_of_elements = data_type_option.number_of_elements;
                    }
                    "format" => {
                        options.format = input.parse::<FieldFormatOption>()?.value();
                    }
                    "modifier" => {
                        options.modifier = input.parse::<syn::LitStr>()?.value();
                    }
                    "name" => {
                        options.name = input.parse::<syn::LitStr>()?.value();
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("Unsupported field attribute: {}", identifier),
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "Unsupported field definition",
                ));
            }
            if !input.is_empty() {
                input.parse::<syn::token::Comma>()?;
            }
        }
        Ok(options)
    }
}

/// Group options.
#[derive(Debug, PartialEq)]
pub struct GroupOptions {
    /// Size condition.
    pub size_condition: Option<String>,

    /// Fields.
    pub fields: Vec<FieldOptions>,
}

impl GroupOptions {
    /// Creates new options.
    pub fn new() -> Self {
        Self {
            size_condition: None,
            fields: Vec::new(),
        }
    }
}

impl Parse for GroupOptions {
    /// Parses the options from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut group_options: GroupOptions = GroupOptions::new();

        while !input.is_empty() {
            if let Ok(ident) = input.fork().parse::<syn::Ident>() {
                let identifier: String = ident.to_string();

                match identifier.as_str() {
                    "size_condition" => {
                        input.parse::<syn::Ident>()?;
                        input.parse::<syn::token::Eq>()?;

                        let mut string_value: String = input.parse::<syn::LitStr>()?.value();
                        if !string_value.is_empty() {
                            string_value = format!("data.len() {}", string_value);
                        }
                        group_options.size_condition = Some(string_value);
                    }
                    "field" => {
                        let meta_list: syn::MetaList = input.parse()?;

                        match syn::parse2::<FieldOptions>(meta_list.tokens.clone()) {
                            Ok(field_options) => group_options.fields.push(field_options),
                            Err(error) => {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!(
                                        "Unable to parse LayoutMap structure field with error: {}",
                                        error
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("Unsupported LayoutMap group attribute: {}", identifier),
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "Unsupported LayoutMap group definition",
                ));
            }
            if !input.is_empty() {
                input.parse::<syn::token::Comma>()?;
            }
        }
        Ok(group_options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_quote;

    #[test]
    fn test_parse_byte_order_option() -> syn::Result<()> {
        let test_struct: ByteOrderOption = syn::parse2(parse_quote! {
            ""
        })?;
        assert_eq!(
            test_struct,
            ByteOrderOption {
                value: ByteOrder::NotSet
            }
        );

        let test_struct: ByteOrderOption = syn::parse2(parse_quote! {
            "big"
        })?;
        assert_eq!(
            test_struct,
            ByteOrderOption {
                value: ByteOrder::BigEndian
            }
        );

        let test_struct: ByteOrderOption = syn::parse2(parse_quote! {
            "little"
        })?;
        assert_eq!(
            test_struct,
            ByteOrderOption {
                value: ByteOrder::LittleEndian
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_data_type_option() -> syn::Result<()> {
        let test_struct: FieldDataTypeOption = syn::parse2(parse_quote! {
            "u8"
        })?;
        assert_eq!(
            test_struct,
            FieldDataTypeOption {
                value: DataType::UnsignedInteger8Bit,
                number_of_elements: 1
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_format_option() -> syn::Result<()> {
        let test_struct: FieldFormatOption = syn::parse2(parse_quote! {
            ""
        })?;
        assert_eq!(
            test_struct,
            FieldFormatOption {
                value: Format::NotSet
            }
        );

        let test_struct: FieldFormatOption = syn::parse2(parse_quote! {
            "char"
        })?;
        assert_eq!(
            test_struct,
            FieldFormatOption {
                value: Format::Character
            }
        );

        let test_struct: FieldFormatOption = syn::parse2(parse_quote! {
            "hex"
        })?;
        assert_eq!(
            test_struct,
            FieldFormatOption {
                value: Format::Hexadecimal
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_options() -> syn::Result<()> {
        let test_struct: FieldOptions = syn::parse2(parse_quote! {
            name = "format_version", data_type = "u16"
        })?;
        assert_eq!(
            test_struct,
            FieldOptions {
                byte_order: ByteOrder::NotSet,
                data_type: DataType::UnsignedInteger16Bit,
                format: Format::NotSet,
                modifier: String::new(),
                name: String::from("format_version"),
                number_of_elements: 1,
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_group_options() -> syn::Result<()> {
        let test_struct: GroupOptions = syn::parse2(parse_quote! {
            size_condition = "> 32",
            field(name = "format_version", data_type = "u16")
        })?;
        assert_eq!(
            test_struct,
            GroupOptions {
                size_condition: Some(String::from("data.len() > 32")),
                fields: vec![FieldOptions {
                    byte_order: ByteOrder::NotSet,
                    data_type: DataType::UnsignedInteger16Bit,
                    format: Format::NotSet,
                    modifier: String::new(),
                    name: String::from("format_version"),
                    number_of_elements: 1,
                }],
            }
        );
        Ok(())
    }
}
