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

#[derive(Debug, Default, PartialEq)]
pub enum PlistObject {
    /// Collection of values without a key.
    Array(Vec<PlistObject>),

    /// Boolean value.
    Boolean(bool),

    /// Binary data value.
    Data(Vec<u8>),

    /// Date and time value.
    DateTime(f64),

    /// Collection of values with key.
    Dictionary(HashMap<String, PlistObject>),

    /// Floating-point value.
    FloatingPoint(f64),

    /// Integer value.
    Integer(i64),

    /// Empty value.
    #[default]
    None,

    /// String value.
    String(String),
}

impl PlistObject {
    /// Retrieves the reference to a wrapped boolean.
    pub fn as_boolean(&self) -> Option<&bool> {
        match *self {
            PlistObject::Boolean(ref boolean) => Some(boolean),
            _ => None,
        }
    }

    /// Retrieves the reference to wrapped bytes.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            PlistObject::Data(ref data) => Some(data),
            _ => None,
        }
    }

    // TODO: add as_date_time

    /// Retrieves the reference to a wrapped floating-point.
    pub fn as_floating_point(&self) -> Option<&f64> {
        match *self {
            PlistObject::FloatingPoint(ref floating_point) => Some(floating_point),
            _ => None,
        }
    }

    /// Retrieves the reference to a wrapped hashmap.
    pub fn as_hashmap(&self) -> Option<&HashMap<String, PlistObject>> {
        match *self {
            PlistObject::Dictionary(ref hashmap) => Some(hashmap),
            _ => None,
        }
    }

    /// Retrieves the reference to a wrapped integer.
    pub fn as_integer(&self) -> Option<&i64> {
        match *self {
            PlistObject::Integer(ref integer) => Some(integer),
            _ => None,
        }
    }

    /// Retrieves the reference to a wrapped string.
    pub fn as_string(&self) -> Option<&String> {
        match *self {
            PlistObject::String(ref string) => Some(string),
            _ => None,
        }
    }

    /// Retrieves the reference to a wrapped vector.
    pub fn as_vector(&self) -> Option<&Vec<PlistObject>> {
        match *self {
            PlistObject::Array(ref vector) => Some(vector),
            _ => None,
        }
    }

    /// Retrieves a bytes value for a specific key.
    pub fn get_bytes_by_key(&self, key: &str) -> Option<&[u8]> {
        let data_object: &PlistObject = match self.get_object_by_key(key) {
            Some(plist_object) => plist_object,
            None => return None,
        };
        data_object.as_bytes()
    }

    /// Retrieves an integer value for a specific key.
    pub fn get_integer_by_key(&self, key: &str) -> Option<&i64> {
        let integer_object: &PlistObject = match self.get_object_by_key(key) {
            Some(plist_object) => plist_object,
            None => return None,
        };
        integer_object.as_integer()
    }

    /// Retrieves an object value for a specific key.
    pub fn get_object_by_key(&self, key: &str) -> Option<&PlistObject> {
        let hashmap: &HashMap<String, PlistObject> = match self.as_hashmap() {
            Some(hashmap) => hashmap,
            None => return None,
        };
        hashmap.get(key)
    }

    /// Retrieves a string value for a specific key.
    pub fn get_string_by_key(&self, key: &str) -> Option<&String> {
        let string_object: &PlistObject = match self.get_object_by_key(key) {
            Some(plist_object) => plist_object,
            None => return None,
        };
        string_object.as_string()
    }

    /// Retrieves a vector value for a specific key.
    pub fn get_vector_by_key(&self, key: &str) -> Option<&Vec<PlistObject>> {
        let array_object: &PlistObject = match self.get_object_by_key(key) {
            Some(plist_object) => plist_object,
            None => return None,
        };
        array_object.as_vector()
    }
}
