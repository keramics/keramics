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

use darling::{FromDeriveInput, FromMeta};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use super::bitmap::BitmapLayout;
use super::enums::{BitOrder, ByteOrder, DataType};
use super::errors::ParseError;
use super::options::{ByteOrderOption, FieldOptions, GroupOptions};
use super::structure::{
    StructureLayout, StructureLayoutBitField, StructureLayoutBitFieldsGroup, StructureLayoutField,
    StructureLayoutGroup, StructureLayoutMember, StructureLayoutSequence,
};

// TODO: add ondemand vector type

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
        self.bit_order.is_empty() && self.data_type.is_empty()
    }

    /// Parses the bit order.
    pub fn parse_bit_order(&self) -> Result<BitOrder, ParseError> {
        if self.bit_order.is_empty() {
            return Err(ParseError::new(String::from("Missing bit order")));
        }
        match self.bit_order.as_str() {
            "msb" | "most" | "MostSignificantBit" => Ok(BitOrder::MostSignificantBit),
            "lsb" | "least" | "LeastSignificantBit" => Ok(BitOrder::LeastSignificantBit),
            _ => Err(ParseError::new(format!(
                "Unsupported bit order: {}",
                self.bit_order
            ))),
        }
    }

    /// Parses the data type.
    fn parse_data_type(&self) -> Result<DataType, ParseError> {
        if self.data_type.is_empty() {
            return Err(ParseError::new(String::from("Missing data type")));
        }
        match self.data_type.as_str() {
            "u8" | "uint8" | "UnsignedInteger8Bit" => Ok(DataType::UnsignedInteger8Bit),
            "u16" | "uint16" | "UnsignedInteger16Bit" => Ok(DataType::UnsignedInteger16Bit),
            "u32" | "uint32" | "UnsignedInteger32Bit" => Ok(DataType::UnsignedInteger32Bit),
            "u64" | "uint64" | "UnsignedInteger64Bit" => Ok(DataType::UnsignedInteger64Bit),
            _ => Err(ParseError::new(format!(
                "Unsupported data type: {}",
                self.data_type
            ))),
        }
    }
}

#[derive(Default, FromMeta)]
#[darling(default)]
struct MethodOptions {
    /// Name.
    name: String,
}

#[derive(Default)]
struct MethodsOptions {
    /// names.
    names: Vec<String>,
}

impl MethodsOptions {
    /// Determines if the options are empty.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl Parse for MethodsOptions {
    /// Parses the options from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let names: Vec<String> =
            Punctuated::<syn::LitStr, syn::token::Comma>::parse_terminated(input)?
                .iter()
                .map(|lit_str| lit_str.value())
                .collect();

        Ok(Self { names })
    }
}

impl FromMeta for MethodsOptions {
    /// Creates the options from the meta item.
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            syn::Meta::List(meta_list) => match syn::parse2::<Self>(meta_list.tokens.clone()) {
                Ok(methods_options) => Ok(methods_options),
                Err(error) => {
                    return Err(darling::Error::custom(format!(
                        "Unable to parse LayoutMap methods with error: {}",
                        error
                    )));
                }
            },
            _ => Err(darling::Error::custom(
                "Unsupported item type for LayoutMap methods",
            )),
        }
    }
}

enum StructureMember {
    BitField(FieldOptions),
    Field(FieldOptions),
    Group(GroupOptions),
}

impl Parse for StructureMember {
    /// Parses the options from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(ident) = input.fork().parse::<syn::Ident>() {
            let identifier: String = ident.to_string();

            match identifier.as_str() {
                "field" => {
                    let meta_list: syn::MetaList = input.parse()?;

                    match syn::parse2::<FieldOptions>(meta_list.tokens.clone()) {
                        Ok(field_options) => {
                            let structure_member: StructureMember = match &field_options.data_type {
                                DataType::BitField8
                                | DataType::BitField16
                                | DataType::BitField32
                                | DataType::BitField64 => StructureMember::BitField(field_options),
                                _ => StructureMember::Field(field_options),
                            };
                            Ok(structure_member)
                        }
                        Err(error) => Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "Unable to parse LayoutMap member field with error: {}",
                                error
                            ),
                        )),
                    }
                }
                "group" => {
                    let meta_list: syn::MetaList = input.parse()?;

                    match syn::parse2::<GroupOptions>(meta_list.tokens.clone()) {
                        Ok(group_options) => Ok(StructureMember::Group(group_options)),
                        Err(error) => Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "Unable to parse LayoutMap member group with error: {}",
                                error
                            ),
                        )),
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unsupported LayoutMap member attribute: {}", identifier),
                    ));
                }
            }
        } else {
            Err(syn::Error::new(
                input.span(),
                "Unsupported LayoutMap member definition",
            ))
        }
    }
}

struct StructureOptions {
    /// Byte order.
    byte_order: ByteOrder,

    /// Members.
    members: Vec<StructureMember>,
}

impl StructureOptions {
    /// Creates new options.
    pub fn new() -> Self {
        Self {
            byte_order: ByteOrder::NotSet,
            members: Vec::new(),
        }
    }

    /// Determines if the options are empty.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

impl Default for StructureOptions {
    /// Creates new options.
    fn default() -> Self {
        Self::new()
    }
}

impl Parse for StructureOptions {
    /// Parses the options from the input.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut options: Self = Self::new();

        while !input.is_empty() {
            if let Ok(ident) = input.fork().parse::<syn::Ident>() {
                let identifier: String = ident.to_string();

                match identifier.as_str() {
                    "byte_order" => {
                        input.parse::<syn::Ident>()?;
                        input.parse::<syn::token::Eq>()?;

                        options.byte_order = input.parse::<ByteOrderOption>()?.value();
                    }
                    "field" => {
                        let meta_list: syn::MetaList = input.parse()?;

                        match syn::parse2::<FieldOptions>(meta_list.tokens.clone()) {
                            Ok(field_options) => {
                                let structure_member: StructureMember =
                                    match &field_options.data_type {
                                        DataType::BitField8
                                        | DataType::BitField16
                                        | DataType::BitField32
                                        | DataType::BitField64 => {
                                            StructureMember::BitField(field_options)
                                        }
                                        _ => StructureMember::Field(field_options),
                                    };
                                options.members.push(structure_member);
                            }
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
                    "group" => {
                        let meta_list: syn::MetaList = input.parse()?;

                        match syn::parse2::<GroupOptions>(meta_list.tokens.clone()) {
                            Ok(group_options) => {
                                options.members.push(StructureMember::Group(group_options))
                            }
                            Err(error) => {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!(
                                        "Unable to parse LayoutMap structure group with error: {}",
                                        error
                                    ),
                                ));
                            }
                        }
                    }
                    // TODO: remove, member has been deprecated
                    "member" => {
                        let meta_list: syn::MetaList = input.parse()?;

                        match syn::parse2::<StructureMember>(meta_list.tokens.clone()) {
                            Ok(structure_member) => options.members.push(structure_member),
                            Err(error) => {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!(
                                        "Unable to parse LayoutMap structure member with error: {}",
                                        error
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("Unsupported LayoutMap structure attribute: {}", identifier),
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "Unsupported LayoutMap structure definition",
                ));
            }
            if !input.is_empty() {
                input.parse::<syn::token::Comma>()?;
            }
        }
        Ok(options)
    }
}

impl FromMeta for StructureOptions {
    /// Creates the options from the meta item.
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            syn::Meta::List(meta_list) => match syn::parse2::<Self>(meta_list.tokens.clone()) {
                Ok(methods_options) => Ok(methods_options),
                Err(error) => {
                    return Err(darling::Error::custom(format!(
                        "Unable to parse LayoutMap structure with error: {}",
                        error
                    )));
                }
            },
            _ => Err(darling::Error::custom(
                "Unsupported item type for LayoutMap structure",
            )),
        }
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(layout_map), supports(struct_named))]
struct LayoutMapOptions {
    /// Bitmap.
    #[darling(default)]
    pub bitmap: BitmapOptions,

    /// Structure.
    #[darling(default)]
    pub structure: StructureOptions,

    // TODO: remove, has been deprecated in favor of methods
    /// Method list.
    #[darling(default, multiple, rename = "method")]
    method_list: Vec<MethodOptions>,

    /// Methods.
    #[darling(default)]
    methods: MethodsOptions,
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
            )));
        }
    };
    let data_type: DataType = match options.bitmap.parse_data_type() {
        Ok(data_type) => data_type,
        Err(error) => {
            return Err(ParseError::new(format!(
                "{} in layout map of {}",
                error, name
            )));
        }
    };
    let bitmap_layout: BitmapLayout = BitmapLayout::new(data_type, bit_order);

    // TODO: add option for value size and byte order
    todo!();

    // Ok(bitmap_layout)
}

/// Parses a structure layout member.
fn parse_structure_layout_member(
    name: &String,
    field_options: &FieldOptions,
) -> Result<StructureLayoutMember, ParseError> {
    match &field_options.data_type {
        DataType::BitField8
        | DataType::BitField16
        | DataType::BitField32
        | DataType::BitField64 => Err(ParseError::new(format!(
            "Unsupported data type of field: {}",
            field_options.name
        ))),
        DataType::ByteString | DataType::Ucs2String | DataType::Utf16String => {
            // TODO: change to StructureLayoutString
            let sequence: StructureLayoutSequence =
                parse_structure_layout_sequence(name, field_options)?;

            Ok(StructureLayoutMember::Sequence(sequence))
        }
        _ => {
            if field_options.number_of_elements == 1 {
                if field_options.name.is_empty() {
                    return Err(ParseError::new(format!(
                        "Name missing in field in layout map of {}",
                        name
                    )));
                }
                let field: StructureLayoutField = StructureLayoutField::new(
                    &field_options.name,
                    &field_options.data_type,
                    &field_options.byte_order,
                    &field_options.modifier,
                    &field_options.format,
                );
                Ok(StructureLayoutMember::Field(field))
            } else {
                let sequence: StructureLayoutSequence =
                    parse_structure_layout_sequence(name, field_options)?;

                Ok(StructureLayoutMember::Sequence(sequence))
            }
        }
    }
}

/// Parses a structure layout sequence.
fn parse_structure_layout_sequence(
    name: &String,
    field_options: &FieldOptions,
) -> Result<StructureLayoutSequence, ParseError> {
    if field_options.name.is_empty() {
        return Err(ParseError::new(format!(
            "Name missing in field in layout map of {}",
            name
        )));
    }
    if !field_options.modifier.is_empty() {
        return Err(ParseError::new(format!(
            "Modifier not supported for sequence field: {} in layout map of {}",
            field_options.name, name
        )));
    }
    let field: StructureLayoutField = StructureLayoutField::new(
        &field_options.name,
        &field_options.data_type,
        &field_options.byte_order,
        &field_options.modifier,
        &field_options.format,
    );
    Ok(StructureLayoutSequence::new(
        field,
        field_options.number_of_elements,
    ))
}

/// Parses a structure layout group.
fn parse_structure_layout_group(
    name: &String,
    group_options: &GroupOptions,
) -> Result<StructureLayoutGroup, ParseError> {
    if group_options.fields.is_empty() {
        return Err(ParseError::new(format!(
            "Missing fields in group in layout map of {}",
            name
        )));
    }
    let condition: &String = match group_options.size_condition.as_ref() {
        Some(string) => string,
        None => {
            return Err(ParseError::new(format!(
                "Missing condition in group in layout map of {}",
                name
            )));
        }
    };
    let mut group: StructureLayoutGroup = StructureLayoutGroup::new(condition);

    for field_options in group_options.fields.iter() {
        if field_options.name.is_empty() {
            return Err(ParseError::new(format!(
                "Name missing in field in layout map of {}",
                name
            )));
        }
        let field: StructureLayoutField = StructureLayoutField::new(
            &field_options.name,
            &field_options.data_type,
            &field_options.byte_order,
            &field_options.modifier,
            &field_options.format,
        );
        group.fields.push(field);
    }
    Ok(group)
}

/// Parses a structure layout.
fn parse_structure_layout(
    struct_ident: &syn::Ident,
    _struct_fields: &syn::Fields,
    options: &LayoutMapOptions,
) -> Result<StructureLayout, ParseError> {
    let name: String = struct_ident.to_string();

    let mut structure_layout: StructureLayout =
        StructureLayout::new(&name, &options.structure.byte_order);

    for structure_member in options.structure.members.iter() {
        match structure_member {
            StructureMember::BitField(field_options) => {
                if field_options.name.is_empty() {
                    return Err(ParseError::new(format!(
                        "Name missing in field in layout map of {}",
                        name
                    )));
                }
                let new_bitfield_group: bool = match structure_layout.members.last() {
                    Some(StructureLayoutMember::BitFields(bitfields_group)) => {
                        bitfields_group.is_full()
                    }
                    _ => true,
                };
                if new_bitfield_group {
                    let bitfields_group: StructureLayoutBitFieldsGroup =
                        StructureLayoutBitFieldsGroup::new(
                            &field_options.data_type,
                            &field_options.byte_order,
                        );
                    structure_layout
                        .members
                        .push(StructureLayoutMember::BitFields(bitfields_group));
                }
                match structure_layout.members.last_mut() {
                    Some(StructureLayoutMember::BitFields(bitfields_group)) => {
                        if !bitfields_group.bitfields.is_empty() {
                            if field_options.data_type != bitfields_group.data_type {
                                return Err(ParseError::new(format!(
                                    "Unsupported data type of field: {} expected BitField{}",
                                    field_options.name, bitfields_group.size
                                )));
                            }
                        }
                        let bitfield: StructureLayoutBitField = StructureLayoutBitField::new(
                            &field_options.name,
                            field_options.number_of_elements,
                            &field_options.modifier,
                            &field_options.format,
                        );
                        if bitfield.number_of_bits > bitfields_group.get_available_size() {
                            return Err(ParseError::new(format!(
                                "Size of field: {} exceeds size of BitField{}",
                                field_options.name, bitfields_group.size
                            )));
                        }
                        bitfields_group.add_bitfield(bitfield);
                    }
                    _ => {
                        return Err(ParseError::new(String::from(
                            "Unable to retrieve last bitfields group",
                        )));
                    }
                }
            }
            StructureMember::Field(field_options) => {
                if field_options.name.is_empty() {
                    return Err(ParseError::new(format!(
                        "Name missing in field in layout map of {}",
                        name
                    )));
                }
                match structure_layout.members.last() {
                    Some(StructureLayoutMember::BitFields(bitfields_group)) => {
                        if !bitfields_group.is_full() {
                            return Err(ParseError::new(format!(
                                "Incomplete bitfields group before field: {} in layout map of: {}",
                                field_options.name, name
                            )));
                        }
                    }
                    _ => {}
                }
                let field_member: StructureLayoutMember =
                    parse_structure_layout_member(&name, &field_options)?;

                structure_layout.members.push(field_member);
            }
            StructureMember::Group(group_options) => {
                let group: StructureLayoutGroup =
                    parse_structure_layout_group(&name, group_options)?;

                structure_layout
                    .members
                    .push(StructureLayoutMember::Group(group));
            }
        }
    }
    Ok(structure_layout)
}

/// Processes input.
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
            panic!("LayoutMap does not support combined bitmap and structure definitions");
        }
        if !options.method_list.is_empty() && !options.methods.is_empty() {
            panic!("LayoutMap does not support combined method and methods definitions");
        }
        let mut methods = quote!();

        if !options.bitmap.is_empty() {
            // TODO: complete bitmap layout support
            let _bitmap_layout: BitmapLayout = match parse_bitmap_layout(&ident, &fields, &options)
            {
                Ok(bitmap_layout) => bitmap_layout,
                Err(error) => panic!("{error:}"),
            };
        } else if !options.structure.is_empty() {
            let structure_layout: StructureLayout =
                match parse_structure_layout(&ident, &fields, &options) {
                    Ok(structure_layout) => structure_layout,
                    Err(error) => panic!("{error:}"),
                };
            let method_names: Vec<&str> = if !options.methods.is_empty() {
                options
                    .methods
                    .names
                    .iter()
                    .map(|string| string.as_str())
                    .collect()
            } else {
                options
                    .method_list
                    .iter()
                    .map(|method_option| method_option.name.as_str())
                    .collect()
            };
            for method_name in method_names.iter() {
                // TODO: check if read_at_position is used without debug_read_data.
                let generated_code = match *method_name {
                    "debug_read_data" => structure_layout.generate_debug_read_data(),
                    "read_at_position" => structure_layout.generate_read_at_position(),
                    _ => panic!(
                        "Unsupported method: {} in layout map of {}",
                        method_name, ident
                    ),
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

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_quote;

    #[test]
    fn test_derive() {
        LayoutMapOptions::from_derive_input(&parse_quote! {
            #[derive(LayoutMap)]
            #[layout_map()]
            struct MyStruct {}
        })
        .unwrap();
    }

    #[test]
    fn test_derive_structure_with_bitfields() {
        LayoutMapOptions::from_derive_input(&parse_quote! {
            #[derive(LayoutMap)]
            #[layout_map(
                structure(
                    byte_order = "little",
                    field(name = "block_size", data_type = "BitField16<12>", modifier = "+ 1"),
                    field(name = "signature", data_type = "BitField16<3>"),
                    field(name = "is_compressed_flag", data_type = "BitField16<1>"),
                )
            )]
            struct MyStruct {}
        })
        .unwrap();
    }

    #[test]
    fn test_derive_structure_with_fields() {
        LayoutMapOptions::from_derive_input(&parse_quote! {
            #[derive(LayoutMap)]
            #[layout_map(
                structure(
                    byte_order = "little",
                    field(name = "format_version", data_type = "u16"),
                    field(name = "number_of_elements", data_type = "u32"),
                )
            )]
            struct MyStruct {}
        })
        .unwrap();
    }

    // TODO: remove, member has been deprecated
    #[test]
    fn test_derive_structure_with_members() {
        LayoutMapOptions::from_derive_input(&parse_quote! {
            #[derive(LayoutMap)]
            #[layout_map(
                structure(
                    byte_order = "little",
                    member(field(name = "format_version", data_type = "u16")),
                    member(field(name = "number_of_elements", data_type = "u32")),
                    member(group(
                        size_condition = ">= 8",
                        field(name = "extra_size", data_type = "u16")
                    )),
                )
            )]
            struct MyStruct {}
        })
        .unwrap();
    }
}
