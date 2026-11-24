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

#[cfg(test)]
use std::io::SeekFrom;
use std::str::Chars;

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[cfg(test)]
use keramics_core::DataStreamReference;
use keramics_core::ErrorTrace;
use keramics_encodings::Base64Context;

/// Maximum LUKS2 JSON metadata area size (4092 KiB).
const LUKS2_MAXIMUM_JSON_METADATA_SIZE: u64 = 4190208;

#[derive(Parser)]
#[grammar = "src/luksde/metadata.pest"]
struct LuksMetadataParser {}

/// Linux Unified Key Setup (LUKS) version 2 metadata configuration object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataConfig {
    /// JSON area size in bytes.
    pub json_size: u64,

    /// Keyslots area size in bytes.
    pub keyslots_size: u64,

    /// Configuration flags.
    pub flags: Vec<String>,
}

impl LuksMetadataConfig {
    /// Creates a new metadata configuration object.
    pub fn new() -> Self {
        Self {
            json_size: 0,
            keyslots_size: 0,
            flags: Vec::new(),
        }
    }
}

/// Linux Unified Key Setup (LUKS) version 2 metadata digest object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataDigest {
    /// Digest identifier.
    pub identifier: String,

    /// Digest type (for example, "pbkdf2").
    pub digest_type: String,

    /// Keyslot identifiers validated by this digest.
    pub keyslots: Vec<String>,

    /// Segment identifiers validated by this digest.
    pub segments: Vec<String>,

    /// Hashing method.
    pub hashing_method: String,

    /// Number of iterations.
    pub number_of_iterations: u32,

    /// Salt.
    pub salt: Vec<u8>,

    /// Expected digest bytes.
    pub digest: Vec<u8>,
}

impl LuksMetadataDigest {
    /// Creates a new metadata digest object.
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            digest_type: String::new(),
            keyslots: Vec::new(),
            segments: Vec::new(),
            hashing_method: String::new(),
            number_of_iterations: 0,
            salt: Vec::new(),
            digest: Vec::new(),
        }
    }
}

/// Linux Unified Key Setup (LUKS) version 2 metadata keyslot object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataKeySlot {
    /// Keyslot identifier.
    pub identifier: String,

    /// Keyslot type (for example, "luks2" or "reencrypt").
    pub keyslot_type: String,

    /// Master key size in bytes.
    pub key_size: usize,

    /// Optional priority (0 = ignore, 1 = normal, 2 = high).
    pub priority: Option<i64>,

    /// Anti-forensic diffuser type (for example, "luks1").
    pub af_type: String,

    /// Anti-forensic diffuser hashing method.
    pub af_hashing_method: String,

    /// Anti-forensic diffuser number of stripes.
    pub af_number_of_stripes: u32,

    /// Keyslot area type (for example, "raw").
    pub area_type: String,

    /// Keyslot area offset in bytes.
    pub area_offset: u64,

    /// Keyslot area size in bytes.
    pub area_size: u64,

    /// Keyslot area encryption specification.
    pub area_encryption: String,

    /// Keyslot area key size in bytes.
    pub area_key_size: usize,

    /// Key derivation function type (for example, "pbkdf2", "argon2i", "argon2id").
    pub kdf_type: String,

    /// PBKDF2 hashing method.
    pub kdf_hashing_method: String,

    /// PBKDF2 number of iterations.
    pub kdf_number_of_iterations: u32,

    /// Argon2 time cost (number of iterations).
    pub kdf_time: u32,

    /// Argon2 memory cost in KiB.
    pub kdf_memory: u32,

    /// Argon2 parallelism (number of threads/lanes).
    pub kdf_cpus: u32,

    /// Key derivation salt.
    pub kdf_salt: Vec<u8>,
}

impl LuksMetadataKeySlot {
    /// Creates a new metadata keyslot object.
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            keyslot_type: String::new(),
            key_size: 0,
            priority: None,
            af_type: String::new(),
            af_hashing_method: String::new(),
            af_number_of_stripes: 0,
            area_type: String::new(),
            area_offset: 0,
            area_size: 0,
            area_encryption: String::new(),
            area_key_size: 0,
            kdf_type: String::new(),
            kdf_hashing_method: String::new(),
            kdf_number_of_iterations: 0,
            kdf_time: 0,
            kdf_memory: 0,
            kdf_cpus: 0,
            kdf_salt: Vec::new(),
        }
    }
}

/// Linux Unified Key Setup (LUKS) version 2 metadata segment object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataSegment {
    /// Segment identifier.
    pub identifier: String,

    /// Segment type (for example, "crypt" or "linear").
    pub segment_type: String,

    /// Segment offset in bytes.
    pub offset: u64,

    /// Segment size in bytes (`None` for "dynamic").
    pub size: Option<u64>,

    /// Initialization vector tweak value.
    pub initialization_vector_tweak: u64,

    /// Segment encryption specification.
    pub encryption: String,

    /// Sector size in bytes.
    pub sector_size: u16,
}

impl LuksMetadataSegment {
    /// Creates a new metadata segment object.
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
            segment_type: String::new(),
            offset: 0,
            size: None,
            initialization_vector_tweak: 0,
            encryption: String::new(),
            sector_size: 0,
        }
    }
}

/// Linux Unified Key Setup (LUKS) Disk Encryption metadata.
pub struct LuksMetadata {
    /// Configuration.
    pub config: LuksMetadataConfig,

    /// Digests.
    pub digests: Vec<LuksMetadataDigest>,

    /// Key slots.
    pub key_slots: Vec<LuksMetadataKeySlot>,

    /// Segments.
    pub segments: Vec<LuksMetadataSegment>,
}

impl LuksMetadata {
    /// Creates new metadata.
    pub fn new() -> Self {
        Self {
            config: LuksMetadataConfig::new(),
            digests: Vec::new(),
            key_slots: Vec::new(),
            segments: Vec::new(),
        }
    }

    /// Decodes a Base64-encoded JSON string value.
    fn decode_base64_string(value: &str) -> Result<Vec<u8>, ErrorTrace> {
        let unescaped: String = Self::unescape_json_string(value);
        let encoded_bytes: &[u8] = unescaped.as_bytes();
        let buffer_size: usize = ((encoded_bytes.len() / 4) * 3) + 3;
        let mut buffer: Vec<u8> = vec![0; buffer_size];

        let mut base64_context: Base64Context = Base64Context::new();
        match base64_context.decode(encoded_bytes, &mut buffer) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode Base64 string");
                return Err(error);
            }
        }
        buffer.truncate(base64_context.data_size);
        Ok(buffer)
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
        let inner_pairs: Pairs<Rule> = token_pair.into_inner();

        for token_pair in inner_pairs {
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

    /// Parses an anti-forensic (diffuser) object.
    fn parse_af_object(
        &mut self,
        inner_pairs: Pairs<Rule>,
        key_slot: &mut LuksMetadataKeySlot,
    ) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::json_property => {
                    let (identifier, value_pair): (&str, Pair<Rule>) =
                        match Self::parse_json_property_pair(token_pair.into_inner()) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse anti-forensic JSON property"
                                );
                                return Err(error);
                            }
                        };
                    match identifier {
                        "hash" => {
                            let hash_string: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse anti-forensic hash property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.af_hashing_method = hash_string.to_lowercase();
                        }
                        "stripes" => {
                            key_slot.af_number_of_stripes = match Self::parse_u32_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse anti-forensic stripes property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "type" => {
                            let type_string: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse anti-forensic type property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.af_type = type_string.to_lowercase();
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported af object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a keyslot area object.
    fn parse_area_object(
        &mut self,
        inner_pairs: Pairs<Rule>,
        key_slot: &mut LuksMetadataKeySlot,
    ) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::json_property => {
                    let (identifier, value_pair): (&str, Pair<Rule>) =
                        match Self::parse_json_property_pair(token_pair.into_inner()) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse area JSON property"
                                );
                                return Err(error);
                            }
                        };
                    match identifier {
                        "encryption" => {
                            let encryption: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse area encryption property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.area_encryption = encryption.to_lowercase();
                        }
                        "key_size" => {
                            key_slot.area_key_size = match Self::parse_usize_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse area key size property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "offset" => {
                            key_slot.area_offset = match Self::parse_u64_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse area offset property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "size" => {
                            key_slot.area_size = match Self::parse_u64_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse area size property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "type" => {
                            let area_type: String = match Self::parse_json_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse area type property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.area_type = area_type.to_lowercase();
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported area object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a config object.
    fn parse_config_object(&mut self, inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::json_property => {
                    let (identifier, value_pair): (&str, Pair<Rule>) =
                        match Self::parse_json_property_pair(token_pair.into_inner()) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse config JSON property"
                                );
                                return Err(error);
                            }
                        };
                    match identifier {
                        "flags" => {
                            self.config.flags = match Self::parse_string_array_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse config flags property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "json_size" => {
                            self.config.json_size = match Self::parse_u64_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse config JSON size property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "keyslots_size" => {
                            self.config.keyslots_size =
                                match Self::parse_u64_string_pair(value_pair) {
                                    Ok(value) => value,
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to parse config keyslots size property"
                                        );
                                        return Err(error);
                                    }
                                };
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported config object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a digest object.
    fn parse_digest_object(
        &mut self,
        identifier: String,
        inner_pairs: Pairs<Rule>,
    ) -> Result<LuksMetadataDigest, ErrorTrace> {
        let mut digest: LuksMetadataDigest = LuksMetadataDigest::new(identifier);

        for token_pair in inner_pairs {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::json_property => {
                    let (property_identifier, value_pair): (&str, Pair<Rule>) =
                        match Self::parse_json_property_pair(token_pair.into_inner()) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse digest JSON property"
                                );
                                return Err(error);
                            }
                        };
                    match property_identifier {
                        "digest" => {
                            let encoded: String = match Self::parse_json_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest value property"
                                    );
                                    return Err(error);
                                }
                            };
                            digest.digest = match Self::decode_base64_string(encoded.as_str()) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to decode digest value"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "hash" => {
                            let hash_string: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest hash property"
                                    );
                                    return Err(error);
                                }
                            };
                            digest.hashing_method = hash_string.to_lowercase();
                        }
                        "iterations" => {
                            digest.number_of_iterations = match Self::parse_u32_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest iterations property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "keyslots" => {
                            digest.keyslots = match Self::parse_string_array_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest keyslots property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "salt" => {
                            let encoded: String = match Self::parse_json_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest salt property"
                                    );
                                    return Err(error);
                                }
                            };
                            digest.salt = match Self::decode_base64_string(encoded.as_str()) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to decode digest salt"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "segments" => {
                            digest.segments = match Self::parse_string_array_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest segments property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "type" => {
                            let digest_type: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse digest type property"
                                    );
                                    return Err(error);
                                }
                            };
                            digest.digest_type = digest_type.to_lowercase();
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported digest object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(digest)
    }

    /// Parses a digests object.
    fn parse_digests_object(&mut self, inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
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
                let digest_identifier: String = property_identifier.to_string();
                let digest: LuksMetadataDigest =
                    match self.parse_digest_object(digest_identifier, token_pair.into_inner()) {
                        Ok(digest) => digest,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse digest object"
                            );
                            return Err(error);
                        }
                    };
                self.digests.push(digest);
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

    /// Parses an i64 integer from a JSON integer token pair.
    fn parse_i64_pair(value_pair: Pair<Rule>) -> Result<i64, ErrorTrace> {
        if value_pair.as_rule() != Rule::json_integer {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported integer value rule: {:?}",
                value_pair.as_rule()
            )));
        }
        let value_string: &str = value_pair.as_str();
        match value_string.parse::<i64>() {
            Ok(value) => Ok(value),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                format!("Unable to convert value: {} to i64", value_string),
                error
            )),
        }
    }

    /// Parses a JSON property into its unquoted key name and value token pair.
    fn parse_json_property_pair<'a>(
        mut inner_pairs: Pairs<'a, Rule>,
    ) -> Result<(&'a str, Pair<'a, Rule>), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property name"));
            }
        };
        let property_identifier: &str = token_pair.as_str().trim_matches('"');

        let value_pair: Pair<Rule> = match inner_pairs.next() {
            Some(value_pair) => value_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property value"));
            }
        };
        Ok((property_identifier, value_pair))
    }

    /// Parses an unescaped string from a JSON string token pair.
    fn parse_json_string_pair(value_pair: Pair<Rule>) -> Result<String, ErrorTrace> {
        if value_pair.as_rule() != Rule::json_string {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported string value rule: {:?}",
                value_pair.as_rule()
            )));
        }
        let trimmed: &str = value_pair.as_str().trim_matches('"');
        Ok(Self::unescape_json_string(trimmed))
    }

    /// Parses a key derivation object.
    fn parse_kdf_object(
        &mut self,
        inner_pairs: Pairs<Rule>,
        key_slot: &mut LuksMetadataKeySlot,
    ) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::json_property => {
                    let (identifier, value_pair): (&str, Pair<Rule>) =
                        match Self::parse_json_property_pair(token_pair.into_inner()) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse kdf JSON property"
                                );
                                return Err(error);
                            }
                        };
                    match identifier {
                        "cpus" => {
                            key_slot.kdf_cpus = match Self::parse_u32_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse kdf cpus property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "hash" => {
                            let hash_string: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse kdf hash property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.kdf_hashing_method = hash_string.to_lowercase();
                        }
                        "iterations" => {
                            key_slot.kdf_number_of_iterations =
                                match Self::parse_u32_pair(value_pair) {
                                    Ok(value) => value,
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to parse kdf iterations property"
                                        );
                                        return Err(error);
                                    }
                                };
                        }
                        "memory" => {
                            key_slot.kdf_memory = match Self::parse_u32_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse kdf memory property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "salt" => {
                            let encoded: String = match Self::parse_json_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse kdf salt property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.kdf_salt = match Self::decode_base64_string(encoded.as_str()) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to decode kdf salt"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "time" => {
                            key_slot.kdf_time = match Self::parse_u32_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse kdf time property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "type" => {
                            let kdf_type: String = match Self::parse_json_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse kdf type property"
                                    );
                                    return Err(error);
                                }
                            };
                            key_slot.kdf_type = kdf_type.to_lowercase();
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported kdf object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a keyslot object.
    fn parse_keyslot_object(
        &mut self,
        identifier: String,
        inner_pairs: Pairs<Rule>,
    ) -> Result<LuksMetadataKeySlot, ErrorTrace> {
        let mut key_slot: LuksMetadataKeySlot = LuksMetadataKeySlot::new(identifier);

        for token_pair in inner_pairs {
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

    /// Parses a keyslot property.
    fn parse_keyslot_property(
        &mut self,
        mut inner_pairs: Pairs<Rule>,
        key_slot: &mut LuksMetadataKeySlot,
    ) -> Result<(), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing property"));
            }
        };
        let rule: Rule = token_pair.as_rule();

        match rule {
            Rule::af_object => match self.parse_af_object(token_pair.into_inner(), key_slot) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to parse anti-forensic object"
                    );
                    return Err(error);
                }
            },
            Rule::area_object => match self.parse_area_object(token_pair.into_inner(), key_slot) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to parse area object");
                    return Err(error);
                }
            },
            Rule::json_property => {
                let (identifier, value_pair): (&str, Pair<Rule>) =
                    match Self::parse_json_property_pair(token_pair.into_inner()) {
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
                        key_slot.key_size = match Self::parse_usize_pair(value_pair) {
                            Ok(value) => value,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse keyslot key size property"
                                );
                                return Err(error);
                            }
                        };
                    }
                    "priority" => {
                        let priority: i64 = match Self::parse_i64_pair(value_pair) {
                            Ok(value) => value,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse keyslot priority property"
                                );
                                return Err(error);
                            }
                        };
                        key_slot.priority = Some(priority);
                    }
                    "type" => {
                        let keyslot_type: String = match Self::parse_json_string_pair(value_pair) {
                            Ok(value) => value,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse keyslot type property"
                                );
                                return Err(error);
                            }
                        };
                        key_slot.keyslot_type = keyslot_type.to_lowercase();
                    }
                    _ => {}
                }
            }
            Rule::kdf_object => match self.parse_kdf_object(token_pair.into_inner(), key_slot) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to parse kdf object");
                    return Err(error);
                }
            },
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported keyslot property rule: {:?}",
                    rule
                )));
            }
        }
        Ok(())
    }

    /// Parses a keyslots object.
    fn parse_keyslots_object(&mut self, inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
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
                let keyslot_identifier: String = property_identifier.to_string();
                let key_slot: LuksMetadataKeySlot =
                    match self.parse_keyslot_object(keyslot_identifier, token_pair.into_inner()) {
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

    /// Parses a segment object.
    fn parse_segment_object(
        &mut self,
        identifier: String,
        inner_pairs: Pairs<Rule>,
    ) -> Result<LuksMetadataSegment, ErrorTrace> {
        let mut segment: LuksMetadataSegment = LuksMetadataSegment::new(identifier);

        for token_pair in inner_pairs {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::json_property => {
                    let (property_identifier, value_pair): (&str, Pair<Rule>) =
                        match Self::parse_json_property_pair(token_pair.into_inner()) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to parse segment JSON property"
                                );
                                return Err(error);
                            }
                        };
                    match property_identifier {
                        "encryption" => {
                            let encryption: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse segment encryption property"
                                    );
                                    return Err(error);
                                }
                            };
                            segment.encryption = encryption.to_lowercase();
                        }
                        "iv_tweak" => {
                            segment.initialization_vector_tweak = match Self::parse_u64_string_pair(
                                value_pair,
                            ) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse segment initialization vector tweak property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "offset" => {
                            segment.offset = match Self::parse_u64_string_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse segment offset property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "sector_size" => {
                            segment.sector_size = match Self::parse_u16_pair(value_pair) {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse segment sector size property"
                                    );
                                    return Err(error);
                                }
                            };
                        }
                        "size" => {
                            let size_string: String = match Self::parse_json_string_pair(value_pair)
                            {
                                Ok(value) => value,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to parse segment size property"
                                    );
                                    return Err(error);
                                }
                            };
                            if size_string == "dynamic" {
                                segment.size = None;
                            } else {
                                let parsed_size: u64 = match size_string.parse::<u64>() {
                                    Ok(size) => size,
                                    Err(error) => {
                                        return Err(keramics_core::error_trace_new_with_error!(
                                            format!(
                                                "Unable to convert segment size: {} to u64",
                                                size_string
                                            ),
                                            error
                                        ));
                                    }
                                };
                                segment.size = Some(parsed_size);
                            }
                        }
                        "type" => {
                            let segment_type: String =
                                match Self::parse_json_string_pair(value_pair) {
                                    Ok(value) => value,
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to parse segment type property"
                                        );
                                        return Err(error);
                                    }
                                };
                            segment.segment_type = segment_type.to_lowercase();
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported segment object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(segment)
    }

    /// Parses a segments object.
    fn parse_segments_object(&mut self, inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
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
                let segment_identifier: String = property_identifier.to_string();
                let segment: LuksMetadataSegment =
                    match self.parse_segment_object(segment_identifier, token_pair.into_inner()) {
                        Ok(segment) => segment,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse segment object"
                            );
                            return Err(error);
                        }
                    };
                self.segments.push(segment);
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

    /// Parses an array of strings from a JSON array token pair.
    fn parse_string_array_pair(value_pair: Pair<Rule>) -> Result<Vec<String>, ErrorTrace> {
        if value_pair.as_rule() != Rule::json_array {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported array value rule: {:?}",
                value_pair.as_rule()
            )));
        }
        let mut strings: Vec<String> = Vec::new();
        for element_pair in value_pair.into_inner() {
            let element_string: String = match Self::parse_json_string_pair(element_pair) {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to parse string array element"
                    );
                    return Err(error);
                }
            };
            strings.push(element_string);
        }
        Ok(strings)
    }

    /// Parses a tokens object.
    fn parse_tokens_object(&mut self, _inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        Ok(())
    }

    /// Parses a top level property.
    fn parse_top_level_property(&mut self, inner_pairs: Pairs<Rule>) -> Result<(), ErrorTrace> {
        for token_pair in inner_pairs {
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

    /// Parses a u16 integer from a JSON integer token pair.
    fn parse_u16_pair(value_pair: Pair<Rule>) -> Result<u16, ErrorTrace> {
        if value_pair.as_rule() != Rule::json_integer {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported integer value rule: {:?}",
                value_pair.as_rule()
            )));
        }
        let value_string: &str = value_pair.as_str();
        match value_string.parse::<u16>() {
            Ok(value) => Ok(value),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                format!("Unable to convert value: {} to u16", value_string),
                error
            )),
        }
    }

    /// Parses a u32 integer from a JSON integer token pair.
    fn parse_u32_pair(value_pair: Pair<Rule>) -> Result<u32, ErrorTrace> {
        if value_pair.as_rule() != Rule::json_integer {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported integer value rule: {:?}",
                value_pair.as_rule()
            )));
        }
        let value_string: &str = value_pair.as_str();
        match value_string.parse::<u32>() {
            Ok(value) => Ok(value),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                format!("Unable to convert value: {} to u32", value_string),
                error
            )),
        }
    }

    /// Parses a u64 integer encoded as a JSON string.
    fn parse_u64_string_pair(value_pair: Pair<Rule>) -> Result<u64, ErrorTrace> {
        let value_string: String = match Self::parse_json_string_pair(value_pair) {
            Ok(value) => value,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to parse u64 string value");
                return Err(error);
            }
        };
        match value_string.parse::<u64>() {
            Ok(value) => Ok(value),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                format!("Unable to convert value: {} to u64", value_string),
                error
            )),
        }
    }

    /// Parses a usize integer from a JSON integer token pair.
    fn parse_usize_pair(value_pair: Pair<Rule>) -> Result<usize, ErrorTrace> {
        if value_pair.as_rule() != Rule::json_integer {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported integer value rule: {:?}",
                value_pair.as_rule()
            )));
        }
        let value_string: &str = value_pair.as_str();
        match value_string.parse::<usize>() {
            Ok(value) => Ok(value),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                format!("Unable to convert value: {} to usize", value_string),
                error
            )),
        }
    }

    /// Reads the metadata from a specific position in a data stream.
    #[cfg(test)]
    // Used for testing.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u64,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        if data_size == 0 || data_size > LUKS2_MAXIMUM_JSON_METADATA_SIZE {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported metadata size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("LuksMetadata", offset, &data, data_size);

        match self.read_data(&data) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read metadata at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                Err(error)
            }
        }
    }

    /// Reads the metadata from a byte buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: u64 = data.len() as u64;
        if data_size == 0 || data_size > LUKS2_MAXIMUM_JSON_METADATA_SIZE {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported metadata size: {} value out of bounds",
                data_size
            )));
        }
        let string: &str = match str::from_utf8(data) {
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
        let trimmed_string: &str = string.trim_end_matches('\0');
        self.parse(trimmed_string)
    }

    /// Unescapes standard JSON string escape sequences.
    fn unescape_json_string(value: &str) -> String {
        if !value.contains('\\') {
            return value.to_string();
        }
        let mut result: String = String::with_capacity(value.len());
        let mut chars: Chars = value.chars();

        while let Some(character) = chars.next() {
            if character == '\\' {
                if let Some(escaped) = chars.next() {
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\u{0008}'),
                        'f' => result.push('\u{000c}'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        other => {
                            result.push('\\');
                            result.push(other);
                        }
                    }
                }
            } else {
                result.push(character);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    #[test]
    fn test_read_text() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "{",
            "  \"keyslots\": {",
            "    \"0\": {",
            "      \"type\": \"luks2\",",
            "      \"key_size\": 32,",
            "      \"priority\": 1,",
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
            // typos:ignore
            "        \"salt\": \"+woyOmaaWcKoJgywldnIH6o9mkA4zwZnsE\\/Y2fvoNdI=\"",
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

        assert_eq!(metadata.config.json_size, 12288);
        assert_eq!(metadata.config.keyslots_size, 262144);

        let number_of_key_slots: usize = metadata.key_slots.len();
        assert_eq!(number_of_key_slots, 1);
        let key_slot: &LuksMetadataKeySlot = &metadata.key_slots[0];
        assert_eq!(key_slot.identifier, "0");
        assert_eq!(key_slot.keyslot_type, "luks2");
        assert_eq!(key_slot.key_size, 32);
        assert_eq!(key_slot.priority, Some(1));
        assert_eq!(key_slot.af_type, "luks1");
        assert_eq!(key_slot.af_number_of_stripes, 4000);
        assert_eq!(key_slot.af_hashing_method, "sha1");
        assert_eq!(key_slot.area_type, "raw");
        assert_eq!(key_slot.area_offset, 32768);
        assert_eq!(key_slot.area_size, 131072);
        assert_eq!(key_slot.area_encryption, "aes-cbc-plain");
        assert_eq!(key_slot.area_key_size, 32);
        assert_eq!(key_slot.kdf_type, "argon2id");
        assert_eq!(key_slot.kdf_time, 10);
        assert_eq!(key_slot.kdf_memory, 1048576);
        assert_eq!(key_slot.kdf_cpus, 4);
        let key_slot_salt_size: usize = key_slot.kdf_salt.len();
        assert_eq!(key_slot_salt_size, 32);

        let number_of_segments: usize = metadata.segments.len();
        assert_eq!(number_of_segments, 1);
        let segment: &LuksMetadataSegment = &metadata.segments[0];
        assert_eq!(segment.identifier, "0");
        assert_eq!(segment.segment_type, "crypt");
        assert_eq!(segment.offset, 294912);
        assert_eq!(segment.size, None);
        assert_eq!(segment.initialization_vector_tweak, 0);
        assert_eq!(segment.encryption, "aes-cbc-plain");
        assert_eq!(segment.sector_size, 4096);

        let number_of_digests: usize = metadata.digests.len();
        assert_eq!(number_of_digests, 1);
        let digest: &LuksMetadataDigest = &metadata.digests[0];
        assert_eq!(digest.identifier, "0");
        assert_eq!(digest.digest_type, "pbkdf2");
        let expected_identifiers: [String; 1] = [String::from("0")];
        assert_eq!(digest.keyslots, expected_identifiers);
        assert_eq!(digest.segments, expected_identifiers);
        assert_eq!(digest.hashing_method, "sha1");
        assert_eq!(digest.number_of_iterations, 723155);
        let digest_salt_size: usize = digest.salt.len();
        assert_eq!(digest_salt_size, 32);
        let digest_size: usize = digest.digest.len();
        assert_eq!(digest_size, 20);

        Ok(())
    }

    #[test]
    fn test_parse_with_additional_properties_and_errors() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "{",
            "  \"keyslots\": {",
            "    \"0\": {",
            "      \"type\": \"luks2\",",
            "      \"key_size\": 32,",
            "      \"priority\": 1,",
            "      \"custom_property\": \"value\"",
            "    }",
            "  },",
            "  \"tokens\": {",
            "    \"type\": \"luks2-keyring\",",
            "    \"key_description\": \"mykey\"",
            "  },",
            "  \"segments\": {",
            "    \"0\": { \"type\": \"crypt\" }",
            "  },",
            "  \"digests\": {",
            "    \"0\": { \"type\": \"pbkdf2\" }",
            "  },",
            "  \"extra_top_level\": true",
            "}",
        );
        let mut metadata: LuksMetadata = LuksMetadata::new();
        metadata.parse(test_data)?;
        let number_of_key_slots: usize = metadata.key_slots.len();
        assert_eq!(number_of_key_slots, 1);

        // Test with invalid JSON syntax.
        let mut metadata: LuksMetadata = LuksMetadata::new();
        let result: Result<(), ErrorTrace> = metadata.parse("{ invalid }");
        assert!(result.is_err());

        // Test with non-integer keyslot identifier.
        let result: Result<(), ErrorTrace> =
            metadata.parse("{ \"keyslots\": { \"abc\": { \"type\": \"luks2\" } } }");
        assert!(result.is_err());

        // Test with non-integer segment identifier.
        let result: Result<(), ErrorTrace> =
            metadata.parse("{ \"segments\": { \"abc\": { \"type\": \"crypt\" } } }");
        assert!(result.is_err());

        // Test with non-integer digest identifier.
        let result: Result<(), ErrorTrace> =
            metadata.parse("{ \"digests\": { \"abc\": { \"type\": \"pbkdf2\" } } }");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let json_bytes: &[u8] = b"{\"keyslots\":{},\"tokens\":{},\"segments\":{},\"digests\":{},\"config\":{\"json_size\":\"12288\",\"keyslots_size\":\"262144\"}}\0\0";
        let data_stream: DataStreamReference = open_fake_data_stream(json_bytes);
        let data_size: u64 = json_bytes.len() as u64;
        let mut metadata: LuksMetadata = LuksMetadata::new();
        metadata.read_at_position(&data_stream, data_size, SeekFrom::Start(0))?;

        assert_eq!(metadata.config.json_size, 12288);
        assert_eq!(metadata.config.keyslots_size, 262144);

        // Test with unsupported data sizes (0 and > LUKS2_MAXIMUM_JSON_METADATA_SIZE).
        let result: Result<(), ErrorTrace> =
            metadata.read_at_position(&data_stream, 0, SeekFrom::Start(0));
        assert!(result.is_err());

        let result: Result<(), ErrorTrace> = metadata.read_at_position(
            &data_stream,
            LUKS2_MAXIMUM_JSON_METADATA_SIZE + 1,
            SeekFrom::Start(0),
        );
        assert!(result.is_err());

        // Test with invalid UTF-8 bytes.
        let invalid_utf8_stream: DataStreamReference = open_fake_data_stream(&[0xff, 0xfe, 0xfd]);
        let result: Result<(), ErrorTrace> =
            metadata.read_at_position(&invalid_utf8_stream, 3, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test with non-ASCII UTF-8 metadata.
        let non_ascii_bytes: &[u8] = b"{ \"extra\": \"\xc3\xa9\" }";
        let non_ascii_size: u64 = non_ascii_bytes.len() as u64;
        let non_ascii_stream: DataStreamReference = open_fake_data_stream(non_ascii_bytes);
        let result: Result<(), ErrorTrace> =
            metadata.read_at_position(&non_ascii_stream, non_ascii_size, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test with invalid JSON in stream.
        let invalid_json_stream: DataStreamReference = open_fake_data_stream(b"not json");
        let result: Result<(), ErrorTrace> =
            metadata.read_at_position(&invalid_json_stream, 8, SeekFrom::Start(0));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut metadata: LuksMetadata = LuksMetadata::new();

        // Test with empty metadata slice.
        let result: Result<(), ErrorTrace> = metadata.read_data(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unescape_json_string() {
        let plain: String = LuksMetadata::unescape_json_string("plain_value");
        assert_eq!(plain, "plain_value");

        let escaped: String =
            LuksMetadata::unescape_json_string("a\\\"b\\\\c\\/d\\be\\ff\\ng\\rh\\ti\\x");
        assert_eq!(escaped, "a\"b\\c/d\u{0008}e\u{000c}f\ng\rh\ti\\x");
    }

    #[test]
    fn test_parse_with_config_flags_and_fixed_segment_size() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "{",
            "  \"config\": {",
            "    \"json_size\": \"12288\",",
            "    \"keyslots_size\": \"262144\",",
            "    \"flags\": [\"allow-discards\", \"submit-from-crypt-cpus\"],",
            "    \"unknown_config\": 1",
            "  },",
            "  \"segments\": {",
            "    \"0\": {",
            "      \"type\": \"crypt\",",
            "      \"offset\": \"32768\",",
            "      \"size\": \"65536\",",
            "      \"iv_tweak\": \"4\",",
            "      \"encryption\": \"aes-xts-plain64\",",
            "      \"sector_size\": 512,",
            "      \"unknown_segment\": true",
            "    }",
            "  },",
            "  \"keyslots\": {",
            "    \"0\": {",
            "      \"type\": \"luks2\",",
            "      \"key_size\": 32,",
            "      \"af\": {",
            "        \"type\": \"luks1\",",
            "        \"stripes\": 4000,",
            "        \"hash\": \"sha256\",",
            "        \"unknown_af\": 0",
            "      },",
            "      \"area\": {",
            "        \"type\": \"raw\",",
            "        \"offset\": \"32768\",",
            "        \"size\": \"131072\",",
            "        \"encryption\": \"aes-xts-plain64\",",
            "        \"key_size\": 32,",
            "        \"unknown_area\": null",
            "      },",
            "      \"kdf\": {",
            "        \"type\": \"pbkdf2\",",
            "        \"hash\": \"sha256\",",
            "        \"iterations\": 1000,",
            "        \"salt\": \"AAAAAAAAAAAAAAAAAAAAAA==\",",
            "        \"unknown_kdf\": false",
            "      }",
            "    }",
            "  },",
            "  \"digests\": {",
            "    \"0\": {",
            "      \"type\": \"pbkdf2\",",
            "      \"keyslots\": [\"0\"],",
            "      \"segments\": [\"0\"],",
            "      \"hash\": \"sha256\",",
            "      \"iterations\": 1000,",
            "      \"salt\": \"AAAAAAAAAAAAAAAAAAAAAA==\",",
            "      \"digest\": \"AAAAAAAAAAAAAAAAAAAAAA==\",",
            "      \"unknown_digest\": []",
            "    }",
            "  }",
            "}",
        );
        let mut metadata: LuksMetadata = LuksMetadata::new();
        metadata.parse(test_data)?;

        let expected_flags: [String; 2] = [
            String::from("allow-discards"),
            String::from("submit-from-crypt-cpus"),
        ];
        assert_eq!(metadata.config.flags, expected_flags);
        assert_eq!(metadata.segments[0].size, Some(65536));
        assert_eq!(metadata.segments[0].initialization_vector_tweak, 4);
        assert_eq!(metadata.key_slots[0].kdf_hashing_method, "sha256");
        assert_eq!(metadata.key_slots[0].kdf_number_of_iterations, 1000);

        Ok(())
    }

    #[test]
    fn test_parse_with_invalid_property_values() {
        let invalid_cases: [&str; 38] = [
            // Invalid config properties.
            "{\"config\":{\"flags\":123}}",
            "{\"config\":{\"flags\":[123]}}",
            "{\"config\":{\"json_size\":12288}}",
            "{\"config\":{\"json_size\":\"not_a_number\"}}",
            "{\"config\":{\"keyslots_size\":\"not_a_number\"}}",
            // Invalid keyslot properties.
            "{\"keyslots\":{\"0\":{\"key_size\":\"32\"}}}",
            "{\"keyslots\":{\"0\":{\"key_size\":-1}}}",
            "{\"keyslots\":{\"0\":{\"priority\":\"1\"}}}",
            "{\"keyslots\":{\"0\":{\"priority\":999999999999999999999}}}",
            "{\"keyslots\":{\"0\":{\"type\":123}}}",
            // Invalid af properties.
            "{\"keyslots\":{\"0\":{\"af\":{\"hash\":123}}}}",
            "{\"keyslots\":{\"0\":{\"af\":{\"stripes\":\"4000\"}}}}",
            "{\"keyslots\":{\"0\":{\"af\":{\"stripes\":-1}}}}",
            "{\"keyslots\":{\"0\":{\"af\":{\"type\":123}}}}",
            // Invalid area properties.
            "{\"keyslots\":{\"0\":{\"area\":{\"encryption\":123}}}}",
            "{\"keyslots\":{\"0\":{\"area\":{\"key_size\":\"32\"}}}}",
            "{\"keyslots\":{\"0\":{\"area\":{\"offset\":\"bad\"}}}}",
            "{\"keyslots\":{\"0\":{\"area\":{\"size\":\"bad\"}}}}",
            "{\"keyslots\":{\"0\":{\"area\":{\"type\":123}}}}",
            // Invalid kdf properties.
            "{\"keyslots\":{\"0\":{\"kdf\":{\"cpus\":\"4\"}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"hash\":123}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"iterations\":\"1000\"}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"memory\":\"64\"}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"salt\":123}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"salt\":\"!!!invalid_base64!!!\"}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"time\":\"1\"}}}}",
            "{\"keyslots\":{\"0\":{\"kdf\":{\"type\":123}}}}",
            // Invalid segment properties.
            "{\"segments\":{\"0\":{\"encryption\":123}}}",
            "{\"segments\":{\"0\":{\"iv_tweak\":\"bad\"}}}",
            "{\"segments\":{\"0\":{\"offset\":\"bad\"}}}",
            "{\"segments\":{\"0\":{\"sector_size\":\"512\"}}}",
            "{\"segments\":{\"0\":{\"sector_size\":99999}}}",
            "{\"segments\":{\"0\":{\"size\":4096}}}",
            "{\"segments\":{\"0\":{\"size\":\"bad\"}}}",
            "{\"segments\":{\"0\":{\"type\":123}}}",
            // Invalid digest properties.
            "{\"digests\":{\"0\":{\"digest\":\"!!!invalid_base64!!!\"}}}",
            "{\"digests\":{\"0\":{\"salt\":\"!!!invalid_base64!!!\"}}}",
            "{\"digests\":{\"0\":{\"type\":123,\"hash\":123,\"iterations\":\"1\",\"keyslots\":1,\"segments\":1,\"digest\":1,\"salt\":1}}}",
        ];
        for case in invalid_cases {
            let mut metadata: LuksMetadata = LuksMetadata::new();
            let result: Result<(), ErrorTrace> = metadata.parse(case);
            assert!(result.is_err(), "Expected error for case: {}", case);
        }
        // Test remaining individual digest property type errors.
        let digest_cases: [&str; 6] = [
            "{\"digests\":{\"0\":{\"digest\":123}}}",
            "{\"digests\":{\"0\":{\"hash\":123}}}",
            "{\"digests\":{\"0\":{\"iterations\":\"10\"}}}",
            "{\"digests\":{\"0\":{\"keyslots\":123}}}",
            "{\"digests\":{\"0\":{\"salt\":123}}}",
            "{\"digests\":{\"0\":{\"segments\":123}}}",
        ];
        for case in digest_cases {
            let mut metadata: LuksMetadata = LuksMetadata::new();
            let result: Result<(), ErrorTrace> = metadata.parse(case);
            assert!(result.is_err(), "Expected error for digest case: {}", case);
        }
    }
}
