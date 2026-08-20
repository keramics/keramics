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

use std::io::SeekFrom;

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use keramics_checksums::ReversedCrc32Context;
use keramics_core::{DataStreamReference, ErrorTrace};

use super::logical_volume::LinuxLvmLogicalVolume;
use super::physical_volume::LinuxLvmPhysicalVolume;
use super::segment::LinuxLvmSegment;
use super::stripe::LinuxLvmStripe;
use super::volume_group::LinuxLvmVolumeGroup;

#[derive(Parser)]
#[grammar = "src/linuxlvm/metadata.pest"]
struct LinuxLvmMetadataParser {}

/// Linux Logical Volume Manager (LVM) metadata.
pub struct LinuxLvmMetadata {
    /// Volume group.
    pub volume_group: Option<LinuxLvmVolumeGroup>,
}

impl LinuxLvmMetadata {
    /// Creates new metadata.
    pub fn new() -> Self {
        Self { volume_group: None }
    }

    /// Parses Linux Logical Volume Manager (LVM) metadata.
    pub fn parse(&mut self, string: &str) -> Result<(), ErrorTrace> {
        let mut iterator: Pairs<Rule> = match LinuxLvmMetadataParser::parse(Rule::metadata, string)
        {
            Ok(iterator) => iterator,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse metadata",
                    error
                ));
            }
        };
        let token_pair: Pair<Rule> = match iterator.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing metadata"));
            }
        };
        let mut inner_pairs: Pairs<Rule> = token_pair.into_inner();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::EOI => {}
                Rule::global_property => {
                    match self.parse_global_property(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse global property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::volume_group => {
                    let volume_group: LinuxLvmVolumeGroup =
                        match self.parse_volume_group(token_pair.into_inner()) {
                            Ok(volume_group) => volume_group,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse volume group"
                                );
                                return Err(error);
                            }
                        };
                    self.volume_group = Some(volume_group);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported metadata rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a global property.
    fn parse_global_property(&self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        _ = inner_pairs;

        // contents - string
        // creation_host - string
        // creation_time - integer
        // description - string
        // version - integer
        Ok(())
    }

    /// Parses a logical volume.
    fn parse_logical_volume(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<LinuxLvmLogicalVolume, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing identifier"));
            }
        };
        let mut logical_volume: LinuxLvmLogicalVolume = LinuxLvmLogicalVolume::new();

        logical_volume.name = token_pair.as_str().to_string();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::logical_volume_property => {
                    match self
                        .parse_logical_volume_property(token_pair.into_inner(), &mut logical_volume)
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse logical volume property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::segment => {
                    let segment: LinuxLvmSegment = match self.parse_segment(token_pair.into_inner())
                    {
                        Ok(segment) => segment,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to parse segment");
                            return Err(error);
                        }
                    };
                    logical_volume.segments.push(segment);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported logical volume rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(logical_volume)
    }

    /// Parses a logical volume property.
    fn parse_logical_volume_property(
        &self,
        mut inner_pairs: Pairs<Rule>,
        logical_volume: &mut LinuxLvmLogicalVolume,
    ) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let property_value: &str = token_pair.as_str();

        // "creation_host" ~ "=" ~ string
        // "creation_time" ~ "=" ~ integer
        // "flags" ~ "=" ~ list_of_values
        // "status" ~ "=" ~ list_of_strings

        match property_identifier {
            "id" => {
                logical_volume.identifier = property_value.trim_matches('"').to_string();
            }
            "segment_count" => {
                logical_volume.number_of_segments = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Parses logical volumes.
    fn parse_logical_volumes(
        &self,
        mut inner_pairs: Pairs<Rule>,
        volume_group: &mut LinuxLvmVolumeGroup,
    ) -> Result<(), ErrorTrace> {
        if !volume_group.logical_volumes.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Logical volumes already set"
            ));
        }
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::logical_volume => {
                    let logical_volume: LinuxLvmLogicalVolume =
                        match self.parse_logical_volume(token_pair.into_inner()) {
                            Ok(logical_volume) => logical_volume,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse logical volume"
                                );
                                return Err(error);
                            }
                        };
                    volume_group.logical_volumes.push(logical_volume);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported logical volumes rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a physical volume.
    fn parse_physical_volume(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<LinuxLvmPhysicalVolume, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing identifier"));
            }
        };
        let mut physical_volume: LinuxLvmPhysicalVolume = LinuxLvmPhysicalVolume::new();

        let name: &str = token_pair.as_str();

        match name.strip_prefix("pv") {
            Some(string) => {
                physical_volume.index = match usize::from_str_radix(string, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to convert identifier to integer",
                            error
                        ));
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!("Unsupported identifier",));
            }
        }
        physical_volume.name = name.to_string();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::physical_volume_property => {
                    match self.parse_physical_volume_property(
                        token_pair.into_inner(),
                        &mut physical_volume,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse physical volume property"
                            );
                            return Err(error);
                        }
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported physical volume rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(physical_volume)
    }

    /// Parses a physical volume property.
    fn parse_physical_volume_property(
        &self,
        mut inner_pairs: Pairs<Rule>,
        physical_volume: &mut LinuxLvmPhysicalVolume,
    ) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let property_value: &str = token_pair.as_str();

        // "device_id" ~ "=" ~ string
        // "device_id_type" ~ "=" ~ string
        // "flags" ~ "=" ~ list_of_values
        // "status" ~ "=" ~ list_of_strings

        match property_identifier {
            "dev_size" => {
                physical_volume.device_size = match u64::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            "device" => {
                physical_volume.device_path = property_value.trim_matches('"').to_string();
            }
            "id" => {
                physical_volume.identifier = property_value.trim_matches('"').to_string();
            }
            "pe_count" => {
                physical_volume.number_of_extents = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            "pe_start" => {
                physical_volume.start_extent = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Parses physical volumes.
    fn parse_physical_volumes(
        &self,
        mut inner_pairs: Pairs<Rule>,
        volume_group: &mut LinuxLvmVolumeGroup,
    ) -> Result<(), ErrorTrace> {
        if !volume_group.physical_volumes.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Physical volumes already set"
            ));
        }
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::physical_volume => {
                    let physical_volume: LinuxLvmPhysicalVolume =
                        match self.parse_physical_volume(token_pair.into_inner()) {
                            Ok(physical_volume) => physical_volume,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse physical volume"
                                );
                                return Err(error);
                            }
                        };
                    volume_group.physical_volumes.push(physical_volume);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported physical volumes rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a segment.
    fn parse_segment(&self, mut inner_pairs: Pairs<Rule>) -> Result<LinuxLvmSegment, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing identifier"));
            }
        };
        let mut segment: LinuxLvmSegment = LinuxLvmSegment::new();

        segment.name = token_pair.as_str().to_string();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::segment_property => {
                    match self.parse_segment_property(token_pair.into_inner(), &mut segment) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse segment property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::stripes => match self.parse_stripes(token_pair.into_inner(), &mut segment) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to parse stripes");
                        return Err(error);
                    }
                },
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported segment rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(segment)
    }

    /// Parses a segment property.
    fn parse_segment_property(
        &self,
        mut inner_pairs: Pairs<Rule>,
        segment: &mut LinuxLvmSegment,
    ) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let property_value: &str = token_pair.as_str();

        // "type" ~ "=" ~ string

        match property_identifier {
            "extent_count" => {
                segment.number_of_extents = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            "start_extent" => {
                segment.start_extent = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            "stripe_count" => {
                segment.number_of_stripes = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            "type" => {
                segment.segment_type = property_value.trim_matches('"').to_string();
            }
            _ => {}
        }
        Ok(())
    }

    /// Parses stripe.
    fn parse_stripe(&self, mut inner_pairs: Pairs<Rule>) -> Result<LinuxLvmStripe, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let mut stripe: LinuxLvmStripe = LinuxLvmStripe::new();

        stripe.physical_volume_name = token_pair.as_str().to_string();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        stripe.start_extent = match u32::from_str_radix(token_pair.as_str(), 10) {
            Ok(integer_value) => integer_value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert start extent to integer",
                    error
                ));
            }
        };
        Ok(stripe)
    }

    /// Parses stripes.
    fn parse_stripes(
        &self,
        mut inner_pairs: Pairs<Rule>,
        segment: &mut LinuxLvmSegment,
    ) -> Result<(), ErrorTrace> {
        if !segment.stripes.is_empty() {
            return Err(keramics_core::error_trace_new!("Stripes already set"));
        }
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::stripe => {
                    let stripe: LinuxLvmStripe = match self.parse_stripe(token_pair.into_inner()) {
                        Ok(stripe) => stripe,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to parse stripe");
                            return Err(error);
                        }
                    };
                    segment.stripes.push(stripe);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported stripes rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a volume group.
    fn parse_volume_group(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<LinuxLvmVolumeGroup, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing identifier"));
            }
        };
        let mut volume_group: LinuxLvmVolumeGroup = LinuxLvmVolumeGroup::new();

        volume_group.name = token_pair.as_str().to_string();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::logical_volumes => {
                    match self.parse_logical_volumes(token_pair.into_inner(), &mut volume_group) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse logical volumes"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::physical_volumes => {
                    match self.parse_physical_volumes(token_pair.into_inner(), &mut volume_group) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse physical volumes"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::volume_group_property => {
                    match self
                        .parse_volume_group_property(token_pair.into_inner(), &mut volume_group)
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse volume group property"
                            );
                            return Err(error);
                        }
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported volume group rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(volume_group)
    }

    /// Parses a volume group property.
    fn parse_volume_group_property(
        &self,
        mut inner_pairs: Pairs<Rule>,
        volume_group: &mut LinuxLvmVolumeGroup,
    ) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let property_value: &str = token_pair.as_str();

        // flags - list_of_values
        // format - string
        // max_lv - integer
        // max_pv - integer
        // status - list_of_strings

        match property_identifier {
            "id" => {
                volume_group.identifier = property_value.trim_matches('"').to_string();
            }
            "extent_size" => {
                volume_group.extent_size = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            "metadata_copies" => {
                volume_group.number_of_metadata_copies =
                    match u32::from_str_radix(property_value, 10) {
                        Ok(integer_value) => integer_value,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                format!(
                                    "Unable to convert proprerty: {} to integer",
                                    property_identifier
                                ),
                                error
                            ));
                        }
                    };
            }
            "seqno" => {
                volume_group.sequence_number = match u32::from_str_radix(property_value, 10) {
                    Ok(integer_value) => integer_value,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to convert proprerty: {} to integer",
                                property_identifier
                            ),
                            error
                        ));
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Reads the metadata from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u64,
        position: SeekFrom,
        checksum: u32,
    ) -> Result<(), ErrorTrace> {
        // Note that 65536 is an arbitrary chosen limit.
        if data_size == 0 || data_size > 65536 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported metadata size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("LinuxLvmMetadata", offset, &data, data_size);

        if checksum != 0 {
            let mut crc32_context: ReversedCrc32Context =
                ReversedCrc32Context::new(0xedb88320, 0xf597a6cf ^ 0xffffffff);

            crc32_context.update(&data);

            let calculated_checksum: u32 = crc32_context.finalize() ^ 0xffffffff;

            if checksum != calculated_checksum {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    checksum, calculated_checksum
                )));
            }
        }
        let string: String = match String::from_utf8(data) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert metadata into UTF-8 string",
                    error
                ));
            }
        };
        if !string.is_ascii() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported non-ASCII metadata"
            ));
        }
        match self.parse(&string.trim_end_matches('\0')) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read metadata at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_text() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "test_volume_group {\n",
            "    id = \"UUSvbG-b0z4-2xaI-BdLf-U1bX-1azu-CCISYT\"\n",
            "    seqno = 3\n",
            "    format = \"lvm2\"\n",
            "    status = [\"RESIZEABLE\", \"READ\", \"WRITE\"]\n",
            "    flags = []\n",
            "    extent_size = 8192\n",
            "    max_lv = 0\n",
            "    max_pv = 0\n",
            "    metadata_copies = 0\n",
            "\n",
            "    physical_volumes {\n",
            "\n",
            "        pv0 {\n",
            "            id = \"d0ln7w-dLee-efiY-s8o9-2H0a-Iqfv-1TOIkK\"\n",
            "            device = \"/dev/loop99\"\n",
            "\n",
            "            device_id_type = \"loop_file\"\n",
            "            device_id = \"/tmp/lvm.raw\"\n",
            "            status = [\"ALLOCATABLE\"]\n",
            "            flags = []\n",
            "            dev_size = 32768\n",
            "            pe_start = 2048\n",
            "            pe_count = 3\n",
            "        }\n",
            "    }\n",
            "\n",
            "    logical_volumes {\n",
            "\n",
            "        test_logical_volume1 {\n",
            "            id = \"B37nLd-TDBw-7Gua-3lWS-qqQq-I8Mc-rCecOh\"\n",
            "            status = [\"READ\", \"WRITE\", \"VISIBLE\"]\n",
            "            flags = []\n",
            "            creation_time = 1787069050\n",
            "            creation_host = \"keramics\"\n",
            "            segment_count = 1\n",
            "\n",
            "            segment1 {\n",
            "                start_extent = 0\n",
            "                extent_count = 1\n",
            "\n",
            "                type = \"striped\"\n",
            "                stripe_count = 1\n",
            "\n",
            "                stripes = [\n",
            "                    \"pv0\", 0\n",
            "                ]\n",
            "            }\n",
            "        }\n",
            "\n",
            "        test_logical_volume2 {\n",
            "            id = \"RZe0Pf-68mY-1S9C-rZX9-2IGa-e0Sp-xsdS6B\"\n",
            "            status = [\"READ\", \"WRITE\", \"VISIBLE\"]\n",
            "            flags = []\n",
            "            creation_time = 1787069050\n",
            "            creation_host = \"keramics\"\n",
            "            segment_count = 1\n",
            "\n",
            "            segment1 {\n",
            "                start_extent = 0\n",
            "                extent_count = 1\n",
            "\n",
            "                type = \"striped\"\n",
            "                stripe_count = 1\n",
            "\n",
            "                stripes = [\n",
            "                    \"pv0\", 1\n",
            "                ]\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "\n",
            "}\n",
            "# Generated by LVM2 version 2.03.38(2) (2025-12-15): Tue Aug 18 18:04:10 2026\n",
            "\n",
            "contents = \"Text Format Volume Group\"\n",
            "version = 1\n",
            "\n",
            "description = \"Write from lvcreate --name test_logical_volume2 -q --size 4m --type linear test_volume_group.\"\n",
            "\n",
            "creation_host = \"keramics\"	# Linux keramics 7.1.8-200.fc44.x86_64 #1 SMP PREEMPT_DYNAMIC Mon Aug 10 03:35:23 UTC 2026 x86_64\n",
            "creation_time = 1787069050	# Tue Aug 18 18:04:10 2026\n",
            "\n",
        );
        let mut metadata: LinuxLvmMetadata = LinuxLvmMetadata::new();
        metadata.parse(test_data)?;

        Ok(())
    }
}
