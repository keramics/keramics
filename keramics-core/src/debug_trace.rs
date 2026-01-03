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

use std::fmt::Display;
use std::sync::Arc;

use super::mediator::Mediator;

/// Debug trace.
pub struct DebugTrace {}

impl DebugTrace {
    /// Prints text.
    #[inline(always)]
    pub fn print<T: Display>(text: T) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(text);
        }
    }

    /// Prints data.
    #[inline(always)]
    pub fn print_data(description: &str, offset: u64, data: &[u8], data_size: usize, group: bool) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!(
                "{} data of size: {} at offset: {} (0x{:08x})\n",
                description, data_size, offset, offset
            ));
            mediator.debug_print_data(&data, group);
        }
    }

    /// Prints a data field.
    #[inline(always)]
    pub fn print_data_field(identifier: &str, data: &[u8]) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!("    {}\n", identifier));
            mediator.debug_print_data(&data, true);
        }
    }

    /// Prints the end of a trace.
    #[inline(always)]
    pub fn print_end() {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print("}\n\n");
        }
    }

    /// Prints a field.
    #[inline(always)]
    pub fn print_field<V: Display>(identifier: &str, value: V) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!("    {}: {},\n", identifier, value));
        }
    }

    /// Prints a structure representation.
    #[inline(always)]
    pub fn print_structure(debug_read_data: fn(&[u8]) -> String, data: &[u8]) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(debug_read_data(data));
        }
    }

    /// Prints a value.
    #[inline(always)]
    pub fn print_value<V: Display>(description: &str, value: V) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!("{}: {}\n", description, value));
        }
    }

    /// Prints the start of a trace.
    #[inline(always)]
    pub fn print_start(identifier: &str) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!("{} {{\n", identifier));
        }
    }
}
