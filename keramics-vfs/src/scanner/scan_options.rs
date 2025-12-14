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

use keramics_core::ErrorTrace;

/// Virtual File System (VFS) scan option element.
#[derive(Debug, PartialEq)]
pub(super) enum VfsScanOptionElement {
    Identifier(String),
    Index(usize),
    IndexRange(usize, usize),
}

/// Virtual File System (VFS) scan option group.
#[derive(Debug, PartialEq)]
pub(super) enum VfsScanOptionGroup {
    All,
    Elements(Vec<VfsScanOptionElement>),
    None,
    NotSet,
}

impl VfsScanOptionGroup {
    /// Checks if the group contains a specific index.
    pub fn contains_index(&self, index: usize) -> bool {
        match self {
            VfsScanOptionGroup::All => true,
            VfsScanOptionGroup::Elements(elements) => {
                for element in elements {
                    match element {
                        VfsScanOptionElement::Index(element_index) => {
                            if *element_index == index {
                                return true;
                            }
                        }
                        VfsScanOptionElement::IndexRange(range_start, range_end) => {
                            if index >= *range_start && index <= *range_end {
                                return true;
                            }
                        }
                        VfsScanOptionElement::Identifier(_) => {}
                    }
                }
                false
            }
            _ => false,
        }
    }
}

/// Virtual File System (VFS) scan options.
pub struct VfsScanOptions {
    /// Partitions to include in scan.
    pub(super) partitions: VfsScanOptionGroup,
}

impl VfsScanOptions {
    /// Creates a new scan options.
    pub fn new() -> Self {
        Self {
            partitions: VfsScanOptionGroup::NotSet,
        }
    }

    /// Parses the partitions scan option from a string.
    pub fn parse_partitions(&mut self, string: &str) -> Result<(), ErrorTrace> {
        let lowercase_string = string.to_lowercase();

        match lowercase_string.as_str() {
            "" => {}
            "all" => self.partitions = VfsScanOptionGroup::All,
            "none" => {
                return Err(keramics_core::error_trace_new!("Unsupported option: none"));
            }
            _ => {
                let mut elements = Vec::new();

                for string_value in lowercase_string.split(',') {
                    let element: VfsScanOptionElement = match string_value.find("..") {
                        Some(string_index) => {
                            let range_start: usize =
                                match usize::from_str_radix(&string_value[0..string_index], 10) {
                                    Ok(integer_value) => integer_value,
                                    Err(error) => {
                                        return Err(keramics_core::error_trace_new_with_error!(
                                            "Unable to parse start of index range",
                                            error
                                        ));
                                    }
                                };
                            let range_end: usize = match usize::from_str_radix(
                                &string_value[string_index + 2..],
                                10,
                            ) {
                                Ok(integer_value) => integer_value,
                                Err(error) => {
                                    return Err(keramics_core::error_trace_new_with_error!(
                                        "Unable to parse end of index range",
                                        error
                                    ));
                                }
                            };
                            VfsScanOptionElement::IndexRange(range_start, range_end)
                        }
                        None => match usize::from_str_radix(string_value, 10) {
                            Ok(integer_value) => VfsScanOptionElement::Index(integer_value),
                            Err(_) => VfsScanOptionElement::Identifier(string_value.to_string()),
                        },
                    };
                    elements.push(element);
                }
                self.partitions = VfsScanOptionGroup::Elements(elements);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_partitions() -> Result<(), ErrorTrace> {
        let mut test_struct: VfsScanOptions = VfsScanOptions::new();

        assert_eq!(test_struct.partitions, VfsScanOptionGroup::NotSet);

        test_struct.parse_partitions("all")?;
        assert_eq!(test_struct.partitions, VfsScanOptionGroup::All);

        test_struct.parse_partitions("1,5")?;
        assert_eq!(
            test_struct.partitions,
            VfsScanOptionGroup::Elements(vec![
                VfsScanOptionElement::Index(1),
                VfsScanOptionElement::Index(5)
            ])
        );

        test_struct.parse_partitions("1..5")?;
        assert_eq!(
            test_struct.partitions,
            VfsScanOptionGroup::Elements(vec![VfsScanOptionElement::IndexRange(1, 5)])
        );

        // TODO: add tests for identifier

        let result: Result<(), ErrorTrace> = test_struct.parse_partitions("none");
        assert!(result.is_err());

        let result: Result<(), ErrorTrace> = test_struct.parse_partitions("X..5");
        assert!(result.is_err());

        let result: Result<(), ErrorTrace> = test_struct.parse_partitions("1..X");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_partitions_contains_index() -> Result<(), ErrorTrace> {
        let mut test_struct: VfsScanOptions = VfsScanOptions::new();

        test_struct.parse_partitions("all")?;
        assert_eq!(test_struct.partitions.contains_index(1), true);
        assert_eq!(test_struct.partitions.contains_index(9), true);

        test_struct.parse_partitions("1..5")?;
        assert_eq!(test_struct.partitions.contains_index(1), true);
        assert_eq!(test_struct.partitions.contains_index(9), false);

        Ok(())
    }
}
