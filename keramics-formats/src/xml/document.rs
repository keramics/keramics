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

use std::io;

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use keramics_core::ErrorTrace;

use super::attribute::XmlAttribute;
use super::element::XmlElement;

#[derive(Parser)]
#[grammar = "src/xml/xml.pest"]
struct XmlParser {}

/// XML document.
pub struct XmlDocument {
    /// The root element.
    pub root_element: Option<XmlElement>,
}

impl XmlDocument {
    /// Creates a new XML document.
    pub fn new() -> Self {
        Self { root_element: None }
    }

    /// Parses a XML document.
    pub fn parse(&mut self, string: &str) -> Result<(), ErrorTrace> {
        let mut iterator: Pairs<Rule> = match XmlParser::parse(Rule::document, string) {
            Ok(iterator) => iterator,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse XML document",
                    error
                ));
            }
        };
        let token_pair: Pair<Rule> = match iterator.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing XML document"));
            }
        };
        let mut inner_pairs: Pairs<Rule> = token_pair.into_inner();

        let mut root_element: Option<XmlElement> = None;

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::element => {
                    root_element = Some(self.parse_element(token_pair.into_inner())?);
                }
                Rule::EOI | Rule::miscellaneous => {}
                Rule::prolog => {
                    // TODO: extact version, encoding and doctype from prolog
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported rule: {:?}",
                        rule
                    )));
                }
            }
        }
        self.root_element = root_element;

        Ok(())
    }

    /// Parses a XML attribute.
    fn parse_attribute(&self, mut inner_pairs: Pairs<Rule>) -> Result<XmlAttribute, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing attribute name"));
            }
        };
        let name: &str = token_pair.as_str();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing attribute value"));
            }
        };
        let value: &str = token_pair.as_str();

        Ok(XmlAttribute::new(name, value))
    }

    /// Parses a XML element.
    fn parse_element(&self, mut inner_pairs: Pairs<Rule>) -> Result<XmlElement, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing element tag"));
            }
        };
        let rule: Rule = token_pair.as_rule();
        let mut xml_element: XmlElement = match rule {
            Rule::element_empty_tag | Rule::element_start_tag => {
                self.parse_element_tag(token_pair.into_inner())?
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported rule: {:?}",
                    rule
                )));
            }
        };
        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::content => {
                    xml_element.value = self.parse_element_content(token_pair.into_inner())?;
                }
                Rule::element_end_tag => {
                    let inner_token_pair: Pair<Rule> = match token_pair.into_inner().next() {
                        Some(token_pair) => token_pair,
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing element name"));
                        }
                    };
                    let name: &str = inner_token_pair.as_str();

                    if name != xml_element.name.as_str() {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Name mismatch between start tag: {} and end tag: {}",
                            xml_element.name, name
                        )));
                    }
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported rule: {:?}",
                        rule
                    )));
                }
            };
        }
        Ok(xml_element)
    }

    /// Parses XML element content.
    fn parse_element_content(&self, mut inner_pairs: Pairs<Rule>) -> Result<String, ErrorTrace> {
        let mut string_parts: Vec<&str> = Vec::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {
                    string_parts.push(token_pair.as_str());
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(string_parts.join(""))
    }

    /// Parses a XML element start or empty tag.
    fn parse_element_tag(&self, mut inner_pairs: Pairs<Rule>) -> Result<XmlElement, ErrorTrace> {
        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing element name"));
            }
        };
        let name: &str = token_pair.as_str();

        let mut xml_element: XmlElement = XmlElement::new(name);

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::attribute => {
                    let xml_attribute: XmlAttribute =
                        self.parse_attribute(token_pair.into_inner())?;
                    xml_element.attributes.push(xml_attribute);
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported rule: {:?}",
                        rule
                    )));
                }
            }
        }
        Ok(xml_element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> Result<(), ErrorTrace> {
        let test_data: String = [
            "<?xml version=\"1.0\"?>",
            "<greeting>Hello, world!</greeting>",
            "",
        ]
        .join("\n");

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data.as_str())?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");

        Ok(())
    }

    #[test]
    fn test_parse_with_doctype() -> Result<(), ErrorTrace> {
        let test_data: String = [
            "<?xml version=\"1.0\"?>",
            "<!DOCTYPE greeting SYSTEM \"hello.dtd\">",
            "<greeting>Hello, world!</greeting>",
            "",
        ]
        .join("\n");

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data.as_str())?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");

        Ok(())
    }

    #[test]
    fn test_parse_with_inline_doctype() -> Result<(), ErrorTrace> {
        let test_data: String = [
            "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>",
            "<!DOCTYPE greeting [",
            "  <!ELEMENT greeting (#PCDATA)>",
            "]>",
            "<greeting>Hello, world!</greeting>",
            "",
        ]
        .join("\n");

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data.as_str())?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");

        Ok(())
    }
}
