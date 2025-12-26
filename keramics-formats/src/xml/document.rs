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

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::element => {
                    self.root_element = match self.parse_element(token_pair.into_inner()) {
                        Ok(element) => Some(element),
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to parse element");
                            return Err(error);
                        }
                    };
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

        inner_pairs.next();

        let token_pair: Pair<Rule> = match inner_pairs.next() {
            Some(token_pair) => token_pair,
            None => {
                return Err(keramics_core::error_trace_new!("Missing attribute value"));
            }
        };
        // TODO: remove quotes from value
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
                match self.parse_element_tag(token_pair.into_inner()) {
                    Ok(element) => element,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to parse element tag");
                        return Err(error);
                    }
                }
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
                    match self.parse_element_content(token_pair.into_inner(), &mut xml_element) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to parse element content"
                            );
                            return Err(error);
                        }
                    }
                }
                Rule::element_end_tag => match token_pair.into_inner().next() {
                    Some(inner_token_pair) => {
                        let name: &str = inner_token_pair.as_str();

                        if name != xml_element.name.as_str() {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Name mismatch between start tag: {} and end tag: {}",
                                xml_element.name, name
                            )));
                        }
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing element name"));
                    }
                },
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
    fn parse_element_content(
        &self,
        mut inner_pairs: Pairs<Rule>,
        xml_element: &mut XmlElement,
    ) -> Result<(), ErrorTrace> {
        let mut string_parts: Vec<&str> = Vec::new();

        while let Some(token_pair) = inner_pairs.next() {
            let rule: Rule = token_pair.as_rule();
            match rule {
                Rule::character_data => {
                    string_parts.push(token_pair.as_str());
                }
                Rule::element => match self.parse_element(token_pair.into_inner()) {
                    Ok(sub_xml_element) => xml_element.sub_elements.push(sub_xml_element),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to parse sub element");
                        return Err(error);
                    }
                },
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported rule: {:?}",
                        rule
                    )));
                }
            }
        }
        if !string_parts.is_empty() {
            xml_element.value = string_parts.join("");
        }
        Ok(())
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
                Rule::attribute => match self.parse_attribute(token_pair.into_inner()) {
                    Ok(xml_attribute) => xml_element.attributes.push(xml_attribute),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to parse attribute");
                        return Err(error);
                    }
                },
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
        let test_data: &str = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<greeting>Hello, world!</greeting>\n",
            "\n"
        );

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data)?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");

        Ok(())
    }

    #[test]
    fn test_parse_with_attribute() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<greeting type=\"hello\">Hello, world!</greeting>\n",
            "\n"
        );

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data)?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");
        assert_eq!(root_element.attributes.len(), 1);

        let attribute: &XmlAttribute = root_element.attributes.get(0).unwrap();
        assert_eq!(attribute.name, "type");
        // TODO: remove quotes from value
        assert_eq!(attribute.value, "\"hello\"");

        Ok(())
    }

    #[test]
    fn test_parse_with_doctype() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<!DOCTYPE greeting SYSTEM \"hello.dtd\">\n",
            "<greeting>Hello, world!</greeting>\n",
            "\n"
        );

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data)?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");

        Ok(())
    }

    #[test]
    fn test_parse_with_inline_doctype() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n",
            "<!DOCTYPE greeting [\n",
            "  <!ELEMENT greeting (#PCDATA)>\n",
            "]>\n",
            "<greeting>Hello, world!</greeting>\n",
            "\n"
        );

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data)?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.value, "Hello, world!");

        Ok(())
    }

    #[test]
    fn test_parse_with_nested_elements() -> Result<(), ErrorTrace> {
        let test_data: &str = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<greeting>\n",
            "    <message>Hello, world!</message>\n",
            "</greeting>\n",
            "\n"
        );

        let mut document: XmlDocument = XmlDocument::new();
        document.parse(test_data)?;

        assert!(document.root_element.is_some());

        let root_element: XmlElement = document.root_element.unwrap();
        assert_eq!(root_element.name, "greeting");
        assert_eq!(root_element.sub_elements.len(), 1);

        let sub_element: &XmlElement = root_element.sub_elements.get(0).unwrap();
        assert_eq!(sub_element.name, "message");
        assert_eq!(sub_element.value, "Hello, world!");

        Ok(())
    }
}
