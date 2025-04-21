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

use std::collections::HashMap;
use std::io;
use std::str::FromStr;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

use encoding::Base64Stream;

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
    pub fn parse(&mut self, string: &str) -> io::Result<()> {
        let mut iterator: Pairs<Rule> = match XmlPlistParser::parse(Rule::plist_document, string) {
            Ok(iterator) => iterator,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        let token_pair: Pair<Rule> = match iterator.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Missing plist document"),
                ))
            }
        };
        let mut root_object: PlistObject = PlistObject::None;

        let mut inner_pairs: Pairs<Rule> = token_pair.into_inner();
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::plist_element => {
                    root_object = self.parse_plist_element(token_pair.into_inner())?;
                }
                Rule::EOI | Rule::miscellaneous | Rule::plist_prolog => {}
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported plist document rule: {:?}", rule),
                    ))
                }
            }
        }
        if root_object != PlistObject::None {
            self.root_object = root_object;
        }
        Ok(())
    }

    /// Parses a XML plist array content.
    fn parse_plist_array_content(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> io::Result<Vec<PlistObject>> {
        let mut array_values: Vec<PlistObject> = Vec::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {}
                Rule::plist_object_element => {
                    let object: PlistObject =
                        self.parse_plist_object_element(token_pair.into_inner())?;
                    array_values.push(object);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported plist array content rule: {:?}", rule),
                    ))
                }
            };
        }
        Ok(array_values)
    }

    /// Parses a XML plist array element.
    fn parse_plist_array_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing array element",
                ));
            }
        };
        let array_values: Vec<PlistObject> =
            self.parse_plist_array_content(token_pair.into_inner())?;

        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported array element",
                ));
            }
            None => {}
        };
        Ok(PlistObject::Array(array_values))
    }

    /// Parses a XML plist dict content.
    fn parse_plist_dict_content(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> io::Result<HashMap<String, PlistObject>> {
        let mut dict_values: HashMap<String, PlistObject> = HashMap::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {}
                Rule::plist_key_and_object_element_pair => {
                    let (key, object): (String, PlistObject) =
                        self.parse_plist_key_and_object_element_pair(token_pair.into_inner())?;
                    dict_values.insert(key, object);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported plist dict content rule: {:?}", rule),
                    ))
                }
            };
        }
        Ok(dict_values)
    }

    /// Parses a XML plist dict element.
    fn parse_plist_dict_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing plist dict element",
                ));
            }
        };
        let dict_values: HashMap<String, PlistObject> =
            self.parse_plist_dict_content(token_pair.into_inner())?;

        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported plist dict element",
                ));
            }
            None => {}
        };
        Ok(PlistObject::Dictionary(dict_values))
    }

    /// Parses a XML plist content.
    fn parse_plist_content(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        let mut object: PlistObject = PlistObject::None;

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {}
                Rule::plist_object_element => {
                    object = self.parse_plist_object_element(token_pair.into_inner())?;
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported plist content rule: {:?}", rule),
                    ))
                }
            };
        }
        Ok(object)
    }
    /// Parses a XML plist data element.
    fn parse_plist_data_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing data element",
                ));
            }
        };
        let encoded_data: &[u8] = token_pair.as_str().as_bytes();

        let mut base64_stream: Base64Stream = Base64Stream::new(encoded_data, 0, true);

        let mut data: Vec<u8> = Vec::new();
        while let Some(byte_value) = base64_stream.get_value()? {
            data.push(byte_value);
        }
        // TODO: check base64 padding

        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported data element",
                ));
            }
            None => {}
        };
        Ok(PlistObject::Data(data))
    }

    /// Parses a XML plist element.
    fn parse_plist_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        // TODO: parser plist version.
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing plist element",
                ));
            }
        };
        let object: PlistObject = self.parse_plist_content(token_pair.into_inner())?;

        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported plist element",
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
    ) -> io::Result<PlistObject> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing floating-point element",
                ));
            }
        };
        let floating_point_value: f64 = match f64::from_str(token_pair.as_str()) {
            Ok(floating_point_value) => floating_point_value,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported floating-point element",
                ));
            }
            None => {}
        };
        Ok(PlistObject::FloatingPoint(floating_point_value))
    }

    /// Parses a XML plist integer element.
    fn parse_plist_integer_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing integer element",
                ));
            }
        };
        let integer_value: i64 = match i64::from_str_radix(token_pair.as_str(), 10) {
            Ok(integer_value) => integer_value,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported integer element",
                ));
            }
            None => {}
        };
        Ok(PlistObject::Integer(integer_value))
    }

    /// Parses a XML plist key and object element pair.
    fn parse_plist_key_and_object_element_pair(
        &self,
        mut inner_pairs: Pairs<Rule>,
    ) -> io::Result<(String, PlistObject)> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing plist key element",
                ));
            }
        };
        let key: String = self.parse_plist_string_element(token_pair.into_inner())?;
        let mut object: PlistObject = PlistObject::None;

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {}
                Rule::plist_object_element => {
                    object = self.parse_plist_object_element(token_pair.into_inner())?;
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported plist key and object rule: {:?}", rule),
                    ))
                }
            };
        }
        if object == PlistObject::None {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Missing plist object element",
            ));
        };
        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported plist key and object element pair",
                ));
            }
            None => {}
        };
        Ok((key, object))
    }

    /// Parses a XML plist object element.
    fn parse_plist_object_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<PlistObject> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing plist object element",
                ));
            }
        };
        let rule: Rule = token_pair.as_rule();

        let object: PlistObject = match rule {
            Rule::plist_array_element => self.parse_plist_array_element(token_pair.into_inner())?,
            Rule::plist_boolean_false_element => PlistObject::Boolean(false),
            Rule::plist_boolean_true_element => PlistObject::Boolean(true),
            Rule::plist_data_element => self.parse_plist_data_element(token_pair.into_inner())?,
            Rule::plist_date_element => {
                // TODO: YYYY '-' MM '-' DD 'T' HH ':' MM ':' SS 'Z'
                todo!();
            }
            Rule::plist_dict_element => self.parse_plist_dict_element(token_pair.into_inner())?,
            Rule::plist_floating_point_element => {
                self.parse_plist_floating_point_element(token_pair.into_inner())?
            }
            Rule::plist_integer_element => {
                self.parse_plist_integer_element(token_pair.into_inner())?
            }
            Rule::plist_string_element => {
                let string_value: String =
                    self.parse_plist_string_element(token_pair.into_inner())?;
                PlistObject::String(string_value)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported plist object rule: {:?}", rule),
                ))
            }
        };
        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported plist object element",
                ));
            }
            None => {}
        };
        Ok(object)
    }

    /// Parses a XML plist string content.
    fn parse_plist_string_content(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<String> {
        let mut string_parts: Vec<&str> = Vec::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {
                    string_parts.push(token_pair.as_str());
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported plist string content rule: {:?}", rule),
                    ))
                }
            };
        }
        Ok(string_parts.join(""))
    }

    /// Parses a XML plist string element.
    fn parse_plist_string_element(&self, mut inner_pairs: Pairs<Rule>) -> io::Result<String> {
        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing string element",
                ));
            }
        };
        let string_value: String = self.parse_plist_string_content(token_pair.into_inner())?;

        inner_pairs.next();

        match inner_pairs.next() {
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported string element",
                ));
            }
            None => {}
        };
        Ok(string_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_xml() -> io::Result<()> {
        let test_data: String = [
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">",
            "<plist version=\"1.0\">",
            "<dict>",
            "    <key>CFBundleInfoDictionaryVersion</key>",
            "    <string>6.0</string>",
            "    <key>band-size</key>",
            "    <integer>8388608</integer>",
            "    <key>bundle-backingstore-version</key>",
            "    <integer>1</integer>",
            "    <key>diskimage-bundle-type</key>",
            "    <string>com.apple.diskimage.sparsebundle</string>",
            "    <key>size</key>",
            "    <integer>102400000</integer>",
            "</dict>",
            "</plist>",
            "",
        ]
        .join("\n");

        let mut plist: XmlPlist = XmlPlist::new();
        plist.parse(test_data.as_str())?;

        let hashmap: &HashMap<String, PlistObject> = plist.root_object.as_hashmap().unwrap();
        assert_eq!(hashmap.len(), 5);

        let string: &String = plist
            .root_object
            .get_string_by_key("CFBundleInfoDictionaryVersion")
            .unwrap();
        assert_eq!(string, "6.0");

        let integer: &i64 = plist.root_object.get_integer_by_key("band-size").unwrap();
        assert_eq!(*integer, 8388608);

        Ok(())
    }

    #[test]
    fn test_read_xml_with_array() -> io::Result<()> {
        let test_data: String = [
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">",
            "<plist version=\"1.0\">",
            "<dict>",
            "    <key>resource-fork</key>",
            "    <dict>",
            "        <key>blkx</key>",
            "        <array>",
            "            <dict>",
            "                <key>Attributes</key>",
            "                <string>0x0050</string>",
            "                <key>CFName</key>",
            "                <string>Protective Master Boot Record (MBR : 0)</string>",
            "                <key>Data</key>",
            "                <data>",
            "                bWlzaAAAAAEAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAA",
            "                AAgIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "                AAIAAAAgQfL6MwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "                AAAAAAACgAAABQAAAAMAAAAAAAAAAAAAAAAAAAABAAAA",
            "                AAAAIA0AAAAAAAAAH/////8AAAAAAAAAAAAAAAEAAAAA",
            "                AAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "                </data>",
            "                <key>ID</key>",
            "                <string>-1</string>",
            "                <key>Name</key>",
            "                <string>Protective Master Boot Record (MBR : 0)</string>",
            "            </dict>",
            "        </array>",
            "    </dict>",
            "</dict>",
            "</plist>",
            "",
        ]
        .join("\n");

        let mut plist: XmlPlist = XmlPlist::new();
        plist.parse(test_data.as_str())?;

        let hashmap: &HashMap<String, PlistObject> = plist.root_object.as_hashmap().unwrap();
        assert_eq!(hashmap.len(), 1);

        let dictionary_object: &PlistObject = hashmap.get("resource-fork").unwrap();
        let hashmap: &HashMap<String, PlistObject> = dictionary_object.as_hashmap().unwrap();

        let array_object: &PlistObject = hashmap.get("blkx").unwrap();
        let vector: &Vec<PlistObject> = array_object.as_vector().unwrap();
        assert_eq!(vector.len(), 1);

        let dictionary_object: &PlistObject = vector.get(0).unwrap();
        let hashmap: &HashMap<String, PlistObject> = dictionary_object.as_hashmap().unwrap();

        let data_object: &PlistObject = hashmap.get("Data").unwrap();
        let data: &[u8] = data_object.as_bytes().unwrap();
        assert_eq!(data.len(), 284);

        Ok(())
    }
}
