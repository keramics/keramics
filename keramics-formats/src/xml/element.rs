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

use super::attribute::XmlAttribute;

/// XML element.
pub struct XmlElement {
    /// Name.
    pub name: String,

    /// Value (or content).
    pub value: String,

    /// Attributes.
    pub attributes: Vec<XmlAttribute>,

    /// Sub elements.
    pub sub_elements: Vec<XmlElement>,
}

impl XmlElement {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: String::new(),
            attributes: Vec::new(),
            sub_elements: Vec::new(),
        }
    }
}
