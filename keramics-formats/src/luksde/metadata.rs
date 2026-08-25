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

use keramics_core::{DataStreamReference, ErrorTrace};

use super::key_slot::LuksKeySlot;

#[derive(Parser)]
#[grammar = "src/luksde/metadata.pest"]
struct LuksMetadataParser {}

/// Linux Unified Key Setup (LUKS) Disk Encryption metadata.
pub struct LuksMetadata {
    /// Key slots.
    key_slots: Vec<LuksKeySlot>,
}

impl LuksMetadata {
    /// Creates new metadata.
    pub fn new() -> Self {
        Self {
            key_slots: Vec::new(),
        }
    }

    /// Parses Linux Unified Key Setup (LUKS) Disk Encryption metadata.
    pub fn parse(&mut self, string: &str) -> Result<(), ErrorTrace> {
        let mut iterator: Pairs<Rule> = match LuksMetadataParser::parse(Rule::metadata, string) {
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
                Rule::top_level_property => {
                    match self.parse_top_level_property(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse top level property"
                            );
                            return Err(error);
                        }
                    }
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

    /// Parses a config object.
    fn parse_config_object(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        Ok(())
    }

    /// Parses a digests property.
    fn parse_digests_property(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str().trim_matches('"');

        match usize::from_str_radix(property_identifier, 10) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    format!(
                        "Unable to convert property identifier: {} to integer",
                        property_identifier
                    ),
                    error
                ));
            }
        }
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let rule: Rule = token_pair.as_rule();

        match rule {
            Rule::digest_object => {
                // TODO: parse.
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported digests property rule: {:?}",
                    rule
                )));
            }
        }
        Ok(())
    }

    /// Parses a digests object.
    fn parse_digests_object(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::digests_property => {
                    match self.parse_digests_property(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse digests property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::json_property => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported digests object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a JSON property.
    fn parse_json_property<'a>(
        &mut self,
        mut inner_pairs: Pairs<'a, Rule>,
    ) -> Result<(&'a str, &'a str), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str().trim_matches('"');

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let property_value: &str = token_pair.as_str();

        Ok((property_identifier, property_value))
    }

    /// Parses a keyslot property.
    fn parse_keyslot_property(
        &mut self,
        mut inner_pairs: Pairs<Rule>,
        key_slot: &mut LuksKeySlot,
    ) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property"));
            }
        };
        let rule: Rule = token_pair.as_rule();

        match rule {
            Rule::af_object => {
                // TODO: parse.
            }
            Rule::area_object => {
                // TODO: parse.
            }
            Rule::json_property => {
                let (identifier, value): (&str, &str) =
                    match self.parse_json_property(token_pair.into_inner()) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse JSON property"
                            );
                            return Err(error);
                        }
                    };
                match identifier {
                    "key_size" => {
                        // json_integer
                    }
                    "priority" => {
                        // json_integer
                    }
                    "type" => {
                        // json_string
                    }
                    _ => {}
                }
            }
            Rule::kdf_object => {
                // TODO: parse.
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported keyslot property rule: {:?}",
                    rule
                )));
            }
        }
        Ok(())
    }

    /// Parses a keyslot object.
    fn parse_keyslot_object(
        &mut self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<LuksKeySlot, ErrorTrace> {
        let mut key_slot: LuksKeySlot = LuksKeySlot::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::keyslot_property => {
                    match self.parse_keyslot_property(token_pair.into_inner(), &mut key_slot) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse keyslot property"
                            );
                            return Err(error);
                        }
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported keyslot object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(key_slot)
    }

    /// Parses a keyslots property.
    fn parse_keyslots_property(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str().trim_matches('"');

        match usize::from_str_radix(property_identifier, 10) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    format!(
                        "Unable to convert property identifier: {} to integer",
                        property_identifier
                    ),
                    error
                ));
            }
        }
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let rule: Rule = token_pair.as_rule();

        match rule {
            Rule::keyslot_object => {
                let key_slot: LuksKeySlot = match self.parse_keyslot_object(token_pair.into_inner())
                {
                    Ok(key_slot) => key_slot,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse keyslot object"
                        );
                        return Err(error);
                    }
                };
                self.key_slots.push(key_slot);
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported keyslots property rule: {:?}",
                    rule
                )));
            }
        }
        Ok(())
    }

    /// Parses a keyslots object.
    fn parse_keyslots_object(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::keyslots_property => {
                    match self.parse_keyslots_property(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse keyslots property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::json_property => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported keyslots object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a segments property.
    fn parse_segments_property(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str().trim_matches('"');

        match usize::from_str_radix(property_identifier, 10) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    format!(
                        "Unable to convert property identifier: {} to integer",
                        property_identifier
                    ),
                    error
                ));
            }
        }
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        let rule: Rule = token_pair.as_rule();

        match rule {
            Rule::segment_object => {
                // TODO: parse.
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segments property rule: {:?}",
                    rule
                )));
            }
        }
        Ok(())
    }

    /// Parses a segments object.
    fn parse_segments_object(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::segments_property => {
                    match self.parse_segments_property(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse segments property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::json_property => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported segments object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a tokens property.
    fn parse_tokens_property(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        Ok(())
    }

    /// Parses a tokens object.
    fn parse_tokens_object(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::tokens_property => {
                    match self.parse_tokens_property(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse tokens property"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::json_property => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported tokens object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a top level property.
    fn parse_top_level_property(&mut self, mut inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::EOI => {}
                Rule::config_object => match self.parse_config_object(token_pair.into_inner()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse config object"
                        );
                        return Err(error);
                    }
                },
                Rule::digests_object => match self.parse_digests_object(token_pair.into_inner()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse digests object"
                        );
                        return Err(error);
                    }
                },
                Rule::keyslots_object => {
                    match self.parse_keyslots_object(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse keyslots object"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::segments_object => {
                    match self.parse_segments_object(token_pair.into_inner()) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse segments object"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::tokens_object => match self.parse_tokens_object(token_pair.into_inner()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse tokens object"
                        );
                        return Err(error);
                    }
                },
                Rule::json_property => {}
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

    /// Reads the metadata from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u64,
        position: SeekFrom,
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

        keramics_core::debug_trace_data!("LuksMetadata", offset, &data, data_size);

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
            "{",
            "  \"keyslots\": {",
            "    \"0\": {",
            "      \"type\": \"luks2\",",
            "      \"key_size\": 32,",
            "      \"af\": {",
            "        \"type\": \"luks1\",",
            "        \"stripes\": 4000,",
            "        \"hash\": \"sha1\"",
            "      },",
            "      \"area\": {",
            "        \"type\": \"raw\",",
            "        \"offset\": \"32768\",",
            "        \"size\": \"131072\",",
            "        \"encryption\": \"aes-cbc-plain\",",
            "        \"key_size\": 32",
            "      },",
            "      \"kdf\": {",
            "        \"type\": \"argon2id\",",
            "        \"time\": 10,",
            "        \"memory\": 1048576,",
            "        \"cpus\": 4,",
            "        \"salt\": \"+woyOmaaWcKoJgywldnIH6o9mkA4zwZnsE/Y2fvoNdI=\"",
            "      }",
            "    }",
            "  },",
            "  \"tokens\": {},",
            "  \"segments\": {",
            "    \"0\": {",
            "      \"type\": \"crypt\",",
            "      \"offset\": \"294912\",",
            "      \"size\": \"dynamic\",",
            "      \"iv_tweak\": \"0\",",
            "      \"encryption\": \"aes-cbc-plain\",",
            "      \"sector_size\": 4096",
            "    }",
            "  },",
            "  \"digests\": {",
            "    \"0\": {",
            "      \"type\": \"pbkdf2\",",
            "      \"keyslots\": [",
            "        \"0\"",
            "      ],",
            "      \"segments\": [",
            "        \"0\"",
            "      ],",
            "      \"hash\": \"sha1\",",
            "      \"iterations\": 723155,",
            "      \"salt\": \"e/qmUMV8V482EW7DKQBmZr8Qi0SfvqXdqd+iJsxc66g=\",",
            "      \"digest\": \"MN/tbH2W1hwu3aonq5dCIfrk3l8=\"",
            "    }",
            "  },",
            "  \"config\": {",
            "    \"json_size\": \"12288\",",
            "    \"keyslots_size\": \"262144\"",
            "  }",
            "}",
            "\n",
        );
        let mut metadata: LuksMetadata = LuksMetadata::new();
        metadata.parse(test_data)?;

        Ok(())
    }
}
