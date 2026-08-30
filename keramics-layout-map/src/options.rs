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
            "ApfsTime" => DataType::ApfsTime,
            "BitField8" => DataType::BitField8,
            "BitField16" => DataType::BitField16,
            "BitField32" => DataType::BitField32,
            "BitField64" => DataType::BitField64,
            "BitField128" => DataType::BitField128,
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
                let (struct_name, struct_size): (&str, &str) =
                    match extended_type_str.split_once(";") {
                        Some((name, size)) => (name.trim(), size.trim()),
                        None => {
                            return Err(syn::Error::new(
                                input.span(),
                                format!("Unsupported Struct definition: {}", extended_type_str),
                            ));
                        }
                    };
                extended_type_str = "";

                DataType::Struct {
                    name: struct_name.to_string(),
                    size: struct_size.parse::<usize>().unwrap(),
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
                                    format!("Unable to parse group field with error: {}", error),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("Unsupported group attribute: {}", identifier),
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "Unsupported group definition",
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
        let test_cases: &[(&str, ByteOrder)] = &[
            ("", ByteOrder::NotSet),
            ("BigEndian", ByteOrder::BigEndian),
            ("be", ByteOrder::BigEndian),
            ("big", ByteOrder::BigEndian),
            ("LittleEndian", ByteOrder::LittleEndian),
            ("le", ByteOrder::LittleEndian),
            ("little", ByteOrder::LittleEndian),
        ];
        for (option_string, expected_byte_order) in test_cases {
            let byte_order_option: ByteOrderOption = syn::parse2(parse_quote! {
                #option_string
            })?;
            assert_eq!(byte_order_option.value, *expected_byte_order);
        }
        Ok(())
    }

    #[test]
    fn test_parse_byte_order_option_with_unsupported_byte_order() {
        let result: syn::Result<ByteOrderOption> = syn::parse2(parse_quote! { "unsupported" });
        assert!(result.is_err());

        let error: syn::Error = result.unwrap_err();
        assert_eq!(
            error.to_string().as_str(),
            "Unsupported byte order: unsupported"
        );
    }

    #[test]
    fn test_parse_field_data_type_option() -> syn::Result<()> {
        let test_cases: &[(&str, DataType, usize)] = &[
            ("u8", DataType::UnsignedInteger8Bit, 1),
            ("uint16", DataType::UnsignedInteger16Bit, 1),
            ("SignedInteger32Bit", DataType::SignedInteger32Bit, 1),
            ("Filetime", DataType::Filetime, 1),
            ("Uuid", DataType::Uuid, 1),
            ("BitField16<12>", DataType::BitField16, 12),
            ("[u64; 4]", DataType::UnsignedInteger64Bit, 4),
            (
                "Struct<MyStruct; 32>",
                DataType::Struct {
                    name: String::from("MyStruct"),
                    size: 32,
                },
                1,
            ),
        ];
        for (option_string, expected_data_type, expected_number_of_elements) in test_cases {
            let field_data_type_option: FieldDataTypeOption = syn::parse2(parse_quote! {
                #option_string
            })?;
            assert_eq!(field_data_type_option.value, *expected_data_type);
            assert_eq!(
                field_data_type_option.number_of_elements,
                *expected_number_of_elements
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse_field_data_type_option_with_unsupported_data_type() {
        let result: syn::Result<FieldDataTypeOption> = syn::parse2(parse_quote! { "unsupported" });
        assert!(result.is_err());

        let error: syn::Error = result.unwrap_err();
        assert_eq!(
            error.to_string().as_str(),
            "Unsupported data type: unsupported"
        );
    }

    #[test]
    fn test_parse_field_data_type_option_with_invalid_struct_definition() {
        let result: syn::Result<FieldDataTypeOption> =
            syn::parse2(parse_quote! { "Struct<MyStruct" });
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_field_data_type_option_with_unsupported_number_of_elements() {
        let result: syn::Result<FieldDataTypeOption> =
            syn::parse2(parse_quote! { "BitField16<string>" });
        assert!(result.is_err());

        let error: syn::Error = result.unwrap_err();
        assert_eq!(
            error.to_string().as_str(),
            "Unsupported number of elements: string in data type: BitField16"
        );
    }

    #[test]
    fn test_parse_field_format_option() -> syn::Result<()> {
        let test_cases: &[(&str, Format)] = &[
            ("", Format::NotSet),
            ("Character", Format::Character),
            ("char", Format::Character),
            ("Hexadecimal", Format::Hexadecimal),
            ("hex", Format::Hexadecimal),
        ];
        for (option_string, expected_format) in test_cases {
            let field_format_option: FieldFormatOption = syn::parse2(parse_quote! {
                #option_string
            })?;
            assert_eq!(field_format_option.value, *expected_format);
        }
        Ok(())
    }

    #[test]
    fn test_parse_field_format_option_with_unsupported_format() {
        let result: syn::Result<FieldFormatOption> = syn::parse2(parse_quote! { "unsupported" });
        assert!(result.is_err());

        let error: syn::Error = result.unwrap_err();
        assert_eq!(
            error.to_string().as_str(),
            "Unsupported format: unsupported"
        );
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

        let test_struct: FieldOptions = syn::parse2(parse_quote! {
            name = "block_size",
            byte_order = "little",
            data_type = "BitField16<12>",
            modifier = "+ 1",
            format = "hex"
        })?;
        assert_eq!(
            test_struct,
            FieldOptions {
                byte_order: ByteOrder::LittleEndian,
                data_type: DataType::BitField16,
                format: Format::Hexadecimal,
                modifier: String::from("+ 1"),
                name: String::from("block_size"),
                number_of_elements: 12,
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_field_options_with_unsupported_attribute() {
        let result: syn::Result<FieldOptions> = syn::parse2(parse_quote! {
            unsupported = "value"
        });
        assert!(result.is_err());

        let error: syn::Error = result.unwrap_err();
        assert_eq!(
            error.to_string().as_str(),
            "Unsupported field attribute: unsupported"
        );
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

        let test_struct: GroupOptions = syn::parse2(parse_quote! {
            size_condition = "",
            field(name = "format_version", data_type = "u16")
        })?;
        assert_eq!(
            test_struct,
            GroupOptions {
                size_condition: Some(String::new()),
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

        let test_struct: GroupOptions = syn::parse2(parse_quote! {
            size_condition = ">= 8",
            field(name = "extra_size", data_type = "u16"),
            field(name = "extra_data", data_type = "u8")
        })?;
        assert_eq!(
            test_struct,
            GroupOptions {
                size_condition: Some(String::from("data.len() >= 8")),
                fields: vec![
                    FieldOptions {
                        byte_order: ByteOrder::NotSet,
                        data_type: DataType::UnsignedInteger16Bit,
                        format: Format::NotSet,
                        modifier: String::new(),
                        name: String::from("extra_size"),
                        number_of_elements: 1,
                    },
                    FieldOptions {
                        byte_order: ByteOrder::NotSet,
                        data_type: DataType::UnsignedInteger8Bit,
                        format: Format::NotSet,
                        modifier: String::new(),
                        name: String::from("extra_data"),
                        number_of_elements: 1,
                    },
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_group_options_with_unsupported_attribute() {
        let result: syn::Result<GroupOptions> = syn::parse2(parse_quote! {
            unsupported = "value"
        });
        assert!(result.is_err());

        let error: syn::Error = result.unwrap_err();
        assert_eq!(
            error.to_string().as_str(),
            "Unsupported group attribute: unsupported"
        );
    }
}
