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

use std::collections::HashMap;
use std::str::FromStr;

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use keramics_core::ErrorTrace;
use keramics_encodings::Base64Stream;

use super::object::PlistObject;

#[derive(Parser)]
#[grammar = "src/plist/xmlplist.pest"]
struct XmlPlistParser {}

/// XML property list (plist).
pub struct XmlPlist {
    /// The root object.
    pub root_object: PlistObject,
}

impl XmlPlist {
    /// Creates a new XML plist.
    pub fn new() -> Self {
        Self {
            root_object: PlistObject::None,
        }
    }

    /// Parses a XML plist.
    pub fn parse(&mut self, string: &str) -> Result<(), ErrorTrace> {
        let mut iterator: Pairs<Rule> = match XmlPlistParser::parse(Rule::plist_document, string) {
            Ok(iterator) => iterator,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse XML plist",
                    error
                ));
            }
        };
        let token_pair: Pair<Rule> = match iterator.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing XML plist document"
                ));
            }
        };
        let mut inner_pairs: Pairs<Rule> = token_pair.into_inner();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::plist_element => {
                    self.root_object = match self.parse_plist_element(token_pair.into_inner()) {
                        Ok(element) => element,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse XML plist document"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::EOI | Rule::miscellaneous | Rule::plist_prolog => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML plist document rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a XML plist array content.
    fn parse_plist_array_content(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<Vec<PlistObject>, ErrorTrace> {
        let mut array_values: Vec<PlistObject> = Vec::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::character_data => {}
                Rule::plist_object_element => {
                    match self.parse_plist_object_element(token_pair.into_inner()) {
                        Ok(object) => array_values.push(object),
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse XML plist array"
                            );
                            return Err(error);
                        }
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML plist array content rule: {:?}",
                        rule
                    )));
                }
            };
        }
        Ok(array_values)
    }

    /// Parses a XML plist array element.
    fn parse_plist_array_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<PlistObject, ErrorTrace> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing array element"));
            }
        };
        let array_values: Vec<PlistObject> =
            match self.parse_plist_array_content(token_pair.into_inner()) {
                Ok(element) => element,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to parse XML plist array element"
                    );
                    return Err(error);
                }
            };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!("Unsupported array element"));
            }
            None => {}
        }
        Ok(PlistObject::Array(array_values))
    }

    /// Parses a XML plist dict content.
    fn parse_plist_dict_content(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<HashMap<String, PlistObject>, ErrorTrace> {
        let mut dict_values: HashMap<String, PlistObject> = HashMap::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::character_data => {}
                Rule::plist_key_and_object_element_pair => {
                    match self.parse_plist_key_and_object_element_pair(token_pair.into_inner()) {
                        Ok((key, object)) => dict_values.insert(key, object),
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse XML plist dict"
                            );
                            return Err(error);
                        }
                    };
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML plist dict content rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(dict_values)
    }

    /// Parses a XML plist dict element.
    fn parse_plist_dict_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<PlistObject, ErrorTrace> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing XML plist dict element"
                ));
            }
        };
        let dict_values: HashMap<String, PlistObject> =
            match self.parse_plist_dict_content(token_pair.into_inner()) {
                Ok(element) => element,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to parse XML plist dict element"
                    );
                    return Err(error);
                }
            };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported XML plist dict element"
                ));
            }
            None => {}
        }
        Ok(PlistObject::Dictionary(dict_values))
    }

    /// Parses a XML plist content.
    fn parse_plist_content(&self, mut inner_pairs: Pairs<Rule>) -> Result<PlistObject, ErrorTrace> {
        let mut object: PlistObject = PlistObject::None;

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::character_data => {}
                Rule::plist_object_element => {
                    object = match self.parse_plist_object_element(token_pair.into_inner()) {
                        Ok(element) => element,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse XML plist content"
                            );
                            return Err(error);
                        }
                    };
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML plist content rule: {:?}",
                        rule
                    )));
                }
            };
        }
        Ok(object)
    }
    /// Parses a XML plist data element.
    fn parse_plist_data_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<PlistObject, ErrorTrace> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data element"));
            }
        };
        let encoded_data: &[u8] = token_pair.as_str().as_bytes();

        let mut base64_stream: Base64Stream = Base64Stream::new(encoded_data, 0, true);

        let mut data: Vec<u8> = Vec::new();

        loop {
            match base64_stream.get_value() {
                Ok(Some(byte_value)) => data.push(byte_value),
                Ok(None) => break,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve decoded byte value"
                    );
                    return Err(error);
                }
            }
        }
        // TODO: check base64 padding

        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!("Unsupported data element"));
            }
            None => {}
        }
        Ok(PlistObject::Data(data))
    }

    /// Parses a XML plist element.
    fn parse_plist_element(&self, mut inner_pairs: Pairs<Rule>) -> Result<PlistObject, ErrorTrace> {
        // TODO: parse XML plist version.
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing XML plist element"));
            }
        };
        let object: PlistObject = match self.parse_plist_content(token_pair.into_inner()) {
            Ok(element) => element,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to parse XML plist element");
                return Err(error);
            }
        };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported XML plist element"
                ));
            }
            None => {}
        };
        Ok(object)
    }

    /// Parses a XML plist floating-point element.
    fn parse_plist_floating_point_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<PlistObject, ErrorTrace> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing floating-point element"
                ));
            }
        };
        let floating_point_value: f64 = match f64::from_str(token_pair.as_str()) {
            Ok(floating_point_value) => floating_point_value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse floating-point value",
                    error
                ));
            }
        };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported floating-point element"
                ));
            }
            None => {}
        }
        Ok(PlistObject::FloatingPoint(floating_point_value))
    }

    /// Parses a XML plist integer element.
    fn parse_plist_integer_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<PlistObject, ErrorTrace> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing integer element"));
            }
        };
        let integer_value: i64 = match i64::from_str_radix(token_pair.as_str(), 10) {
            Ok(integer_value) => integer_value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse integer value",
                    error
                ));
            }
        };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported integer element"
                ));
            }
            None => {}
        }
        Ok(PlistObject::Integer(integer_value))
    }

    /// Parses a XML plist key and object element pair.
    fn parse_plist_key_and_object_element_pair(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<(String, PlistObject), ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing XML plist key element"
                ));
            }
        };
        let key: String = match self.parse_plist_string_element(token_pair.into_inner()) {
            Ok(element) => element,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to parse XML plist key");
                return Err(error);
            }
        };
        let mut object: PlistObject = PlistObject::None;

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::character_data => {}
                Rule::plist_object_element => {
                    object = match self.parse_plist_object_element(token_pair.into_inner()) {
                        Ok(element) => element,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse XML plist object"
                            );
                            return Err(error);
                        }
                    };
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML plist key and object rule: {:?}",
                        rule
                    )));
                }
            }
        }
        if object == PlistObject::None {
            return Err(keramics_core::error_trace_new!(
                "Missing XML plist object element"
            ));
        };
        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported XML plist key and object element pair"
                ));
            }
            None => {}
        }
        Ok((key, object))
    }

    /// Parses a XML plist object element.
    fn parse_plist_object_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<PlistObject, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing XML plist object element"
                ));
            }
        };
        let rule: Rule = token_pair.as_rule();

        let object: PlistObject = match rule {
            Rule::plist_array_element => {
                match self.parse_plist_array_element(token_pair.into_inner()) {
                    Ok(element) => element,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse array object"
                        );
                        return Err(error);
                    }
                }
            }
            Rule::plist_boolean_false_element => PlistObject::Boolean(false),
            Rule::plist_boolean_true_element => PlistObject::Boolean(true),
            Rule::plist_data_element => {
                match self.parse_plist_data_element(token_pair.into_inner()) {
                    Ok(element) => element,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to parse data object");
                        return Err(error);
                    }
                }
            }
            Rule::plist_date_element => {
                // TODO: YYYY '-' MM '-' DD 'T' HH ':' MM ':' SS 'Z'
                todo!();
            }
            Rule::plist_dict_element => {
                match self.parse_plist_dict_element(token_pair.into_inner()) {
                    Ok(element) => element,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to parse dict object");
                        return Err(error);
                    }
                }
            }
            Rule::plist_floating_point_element => {
                match self.parse_plist_floating_point_element(token_pair.into_inner()) {
                    Ok(element) => element,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse floating-point object"
                        );
                        return Err(error);
                    }
                }
            }
            Rule::plist_integer_element => {
                match self.parse_plist_integer_element(token_pair.into_inner()) {
                    Ok(element) => element,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to parse integer object"
                        );
                        return Err(error);
                    }
                }
            }
            Rule::plist_string_element => {
                let string_value: String =
                    match self.parse_plist_string_element(token_pair.into_inner()) {
                        Ok(element) => element,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse string object"
                            );
                            return Err(error);
                        }
                    };
                PlistObject::String(string_value)
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported XML plist object rule: {:?}",
                    rule
                )));
            }
        };
        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported XML plist object element"
                ));
            }
            None => {}
        }
        Ok(object)
    }

    /// Parses a XML plist string content.
    fn parse_plist_string_content(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<String, ErrorTrace> {
        let mut string_parts: Vec<&str> = Vec::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();

            match rule {
                Rule::character_data => {
                    string_parts.push(token_pair.as_str());
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported XML plist string content rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(string_parts.join(""))
    }

    /// Parses a XML plist string element.
    fn parse_plist_string_element(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> Result<String, ErrorTrace> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing XML plist string element"
                ));
            }
        };
        let string_value: String = match self.parse_plist_string_content(token_pair.into_inner()) {
            Ok(element) => element,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to parse XML plist string");
                return Err(error);
            }
        };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported XML plist string element"
                ));
            }
            None => {}
        }
        Ok(string_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_xml() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
            "<plist version=\"1.0\">\n",
            "<dict>\n",
            "    <key>CFBundleInfoDictionaryVersion</key>\n",
            "    <string>6.0</string>\n",
            "    <key>band-size</key>\n",
            "    <integer>8388608</integer>\n",
            "    <key>bundle-backingstore-version</key>\n",
            "    <integer>1</integer>\n",
            "    <key>diskimage-bundle-type</key>\n",
            "    <string>com.apple.diskimage.sparsebundle</string>\n",
            "    <key>size</key>\n",
            "    <integer>102400000</integer>\n",
            "</dict>\n",
            "</plist>\n",
            "\n"
        );

        let mut xml_plist: XmlPlist = XmlPlist::new();
        xml_plist.parse(test_data)?;

        let hashmap: &HashMap<String, PlistObject> = xml_plist.root_object.as_hashmap().unwrap();
        assert_eq!(hashmap.len(), 5);

        let string: Option<&String> = xml_plist
            .root_object
            .get_string_by_key("CFBundleInfoDictionaryVersion");
        assert_eq!(string, Some(String::from("6.0")).as_ref());

        let integer: Option<&i64> = xml_plist.root_object.get_integer_by_key("band-size");
        assert_eq!(integer, Some(8388608).as_ref());

        Ok(())
    }

    #[test]
    fn test_read_xml_with_array() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
            "<plist version=\"1.0\">\n",
            "<dict>\n",
            "    <key>resource-fork</key>\n",
            "    <dict>\n",
            "        <key>blkx</key>\n",
            "        <array>\n",
            "            <dict>\n",
            "                <key>Attributes</key>\n",
            "                <string>0x0050</string>\n",
            "                <key>CFName</key>\n",
            "                <string>Protective Master Boot Record (MBR : 0)</string>\n",
            "                <key>Data</key>\n",
            "                <data>\n",
            "                bWlzaAAAAAEAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAA\n",
            "                AAgIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
            "                AAIAAAAgQfL6MwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
            "                AAAAAAACgAAABQAAAAMAAAAAAAAAAAAAAAAAAAABAAAA\n",
            "                AAAAIA0AAAAAAAAAH/////8AAAAAAAAAAAAAAAEAAAAA\n",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAA=\n",
            "                </data>\n",
            "                <key>ID</key>\n",
            "                <string>-1</string>\n",
            "                <key>Name</key>\n",
            "                <string>Protective Master Boot Record (MBR : 0)</string>\n",
            "            </dict>\n",
            "        </array>\n",
            "    </dict>\n",
            "</dict>\n",
            "</plist>\n",
            "\n"
        );

        let mut xml_plist: XmlPlist = XmlPlist::new();
        xml_plist.parse(test_data)?;

        let hashmap: &HashMap<String, PlistObject> = xml_plist.root_object.as_hashmap().unwrap();
        assert_eq!(hashmap.len(), 1);

        let dictionary_object: &PlistObject = hashmap.get("resource-fork").unwrap();
        let hashmap: &HashMap<String, PlistObject> = dictionary_object.as_hashmap().unwrap();

        let array_object: &PlistObject = hashmap.get("blkx").unwrap();
        let slice: &[PlistObject] = array_object.as_slice().unwrap();
        assert_eq!(slice.len(), 1);

        let dictionary_object: &PlistObject = slice.get(0).unwrap();
        let hashmap: &HashMap<String, PlistObject> = dictionary_object.as_hashmap().unwrap();

        let data_object: &PlistObject = hashmap.get("Data").unwrap();
        let data: &[u8] = data_object.as_bytes().unwrap();
        assert_eq!(data.len(), 284);

        Ok(())
    }
}
