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

use std::fmt::{Display, Write};
use std::sync::Arc;

use super::formatters::format_as_hexdump;
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
            mediator.debug_print(format_as_hexdump(&data, group));
        }
    }

    /// Prints a data field.
    #[inline(always)]
    pub fn print_data_field(identifier: &str, data: &[u8]) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!("    {}\n", identifier));
            mediator.debug_print(format_as_hexdump(&data, true));
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

    /// Prints the start of a trace.
    #[inline(always)]
    pub fn print_start(identifier: &str) {
        let mediator: Arc<Mediator> = Mediator::current();

        if mediator.debug_output {
            mediator.debug_print(format!("{} {{\n", identifier));
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

    /// Creates a debug trace scope that caches output.
    #[inline]
    pub fn scope<F, R, E>(function: F) -> Result<R, E>
    where
        F: FnOnce(&mut DebugTraceScope) -> Result<R, E>,
    {
        let mediator = Mediator::current();
        let debug_output: bool = mediator.debug_output;

        let mut scope = DebugTraceScope {
            mediator: &mediator,
            debug_output,
            output: if debug_output {
                String::with_capacity(1024)
            } else {
                String::new()
            },
        };
        let result = function(&mut scope);

        if debug_output && !scope.output.is_empty() {
            scope.mediator.debug_print(scope.output);
        }
        result
    }

    /// Creates a debug trace scope.
    #[inline]
    pub fn static_scope<F>(function: F)
    where
        F: FnOnce(&DebugTraceStaticScope),
    {
        let mediator = Mediator::current();

        if mediator.debug_output {
            let scope = DebugTraceStaticScope {
                mediator: &mediator,
            };
            function(&scope);
        }
    }
}

/// Debug trace scope that caches output.
pub struct DebugTraceScope<'a> {
    /// Mediator.
    mediator: &'a Mediator,

    /// Debug output.
    pub debug_output: bool,

    /// Output.
    pub output: String,
}

impl<'a> DebugTraceScope<'a> {
    /// Prints text.
    #[inline(always)]
    pub fn print<T: Display>(&mut self, text: T) {
        if self.debug_output {
            _ = write!(self.output, "{}", text);
        }
    }

    /// Prints data.
    #[inline(always)]
    pub fn print_data(
        &mut self,
        description: &str,
        offset: u64,
        data: &[u8],
        data_size: usize,
        group: bool,
    ) {
        if self.debug_output {
            _ = write!(
                self.output,
                "{} data of size: {} at offset: {} (0x{:08x})\n",
                description, data_size, offset, offset
            );
            self.output
                .push_str(format_as_hexdump(&data, group).as_str());
        }
    }

    /// Prints a data field.
    #[inline(always)]
    pub fn print_data_field(&mut self, identifier: &str, data: &[u8]) {
        if self.debug_output {
            _ = write!(self.output, "    {}\n", identifier);
            self.output
                .push_str(format_as_hexdump(&data, true).as_str());
        }
    }

    /// Prints the end of a trace.
    #[inline(always)]
    pub fn print_end(&mut self) {
        if self.debug_output {
            _ = write!(self.output, "}}\n\n");
        }
    }

    /// Prints a field.
    #[inline(always)]
    pub fn print_field<V: Display>(&mut self, identifier: &str, value: V) {
        if self.debug_output {
            _ = write!(self.output, "    {}: {},\n", identifier, value);
        }
    }

    /// Prints the start of a trace.
    #[inline(always)]
    pub fn print_start(&mut self, identifier: &str) {
        if self.debug_output {
            _ = write!(self.output, "{} {{\n", identifier);
        }
    }

    /// Prints a structure representation.
    #[inline(always)]
    pub fn print_structure(&mut self, debug_read_data: fn(&[u8]) -> String, data: &[u8]) {
        if self.debug_output {
            self.output.push_str(&debug_read_data(data));
        }
    }

    /// Prints a value.
    #[inline(always)]
    pub fn print_value<V: Display>(&mut self, description: &str, value: V) {
        if self.debug_output {
            _ = write!(self.output, "{}: {}\n", description, value);
        }
    }
}

/// Debug trace scope.
pub struct DebugTraceStaticScope<'a> {
    /// Mediator.
    mediator: &'a Mediator,
}

impl<'a> DebugTraceStaticScope<'a> {
    /// Prints text.
    #[inline(always)]
    pub fn print<T: Display>(&self, text: T) {
        self.mediator.debug_print(text);
    }

    /// Prints data.
    #[inline(always)]
    pub fn print_data(
        &self,
        description: &str,
        offset: u64,
        data: &[u8],
        data_size: usize,
        group: bool,
    ) {
        self.mediator.debug_print(format!(
            "{} data of size: {} at offset: {} (0x{:08x})\n",
            description, data_size, offset, offset
        ));
        self.mediator.debug_print(format_as_hexdump(&data, group));
    }

    /// Prints a data field.
    #[inline(always)]
    pub fn print_data_field(&self, identifier: &str, data: &[u8]) {
        self.mediator.debug_print(format!("    {}\n", identifier));
        self.mediator.debug_print(format_as_hexdump(&data, true));
    }

    /// Prints the end of a trace.
    #[inline(always)]
    pub fn print_end(&self) {
        self.mediator.debug_print("}\n\n");
    }

    /// Prints a field.
    #[inline(always)]
    pub fn print_field<V: Display>(&self, identifier: &str, value: V) {
        self.mediator
            .debug_print(format!("    {}: {},\n", identifier, value));
    }

    /// Prints the start of a trace.
    #[inline(always)]
    pub fn print_start(&self, identifier: &str) {
        self.mediator.debug_print(format!("{} {{\n", identifier));
    }

    /// Prints a structure representation.
    #[inline(always)]
    pub fn print_structure(&self, debug_read_data: fn(&[u8]) -> String, data: &[u8]) {
        self.mediator.debug_print(debug_read_data(data));
    }

    /// Prints a value.
    #[inline(always)]
    pub fn print_value<V: Display>(&self, description: &str, value: V) {
        self.mediator
            .debug_print(format!("{}: {}\n", description, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;
    use std::sync::Arc;

    use crate::errors::ErrorTrace;
    use crate::formatters::format_as_hexdump;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad,
            0xbe, 0xef,
        ]
    }

    fn test_debug_read_data(data: &[u8]) -> String {
        format!("structure (size: {})", data.len())
    }

    #[test]
    fn test_print_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        DebugTrace::print("test\n");
        DebugTrace::print(String::from("test\n"));
        DebugTrace::print(42);
    }

    #[test]
    fn test_print_with_debug_output_disabled() {
        DebugTrace::print("test\n");
        DebugTrace::print(String::from("test\n"));
        DebugTrace::print(42);
    }

    #[test]
    fn test_print_data_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        let test_data: Vec<u8> = get_test_data();
        DebugTrace::print_data("test", 0, &test_data, 16, false);
        DebugTrace::print_data("test", 0, &test_data, 16, true);
    }

    #[test]
    fn test_print_data_with_debug_output_disabled() {
        let test_data: Vec<u8> = get_test_data();
        DebugTrace::print_data("test", 0, &test_data, 16, false);
        DebugTrace::print_data("test", 0, &test_data, 16, true);
    }

    #[test]
    fn test_print_data_field_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        let test_data: Vec<u8> = get_test_data();
        DebugTrace::print_data_field("test", &test_data);
    }

    #[test]
    fn test_print_data_field_with_debug_output_disabled() {
        let test_data: Vec<u8> = get_test_data();
        DebugTrace::print_data_field("test", &test_data);
    }

    #[test]
    fn test_print_end_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        DebugTrace::print_end();
    }

    #[test]
    fn test_print_end_with_debug_output_disabled() {
        DebugTrace::print_end();
    }

    #[test]
    fn test_print_field_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        DebugTrace::print_field("identifier", "value");
        DebugTrace::print_field("identifier", 42);
    }

    #[test]
    fn test_print_field_with_debug_output_disabled() {
        DebugTrace::print_field("identifier", "value");
        DebugTrace::print_field("identifier", 42);
    }

    #[test]
    fn test_print_start_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        DebugTrace::print_start("test");
    }

    #[test]
    fn test_print_start_with_debug_output_disabled() {
        DebugTrace::print_start("test");
    }

    #[test]
    fn test_print_structure_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        let test_data: Vec<u8> = get_test_data();
        DebugTrace::print_structure(test_debug_read_data, &test_data);
    }

    #[test]
    fn test_print_structure_with_debug_output_disabled() {
        let test_data: Vec<u8> = get_test_data();
        DebugTrace::print_structure(test_debug_read_data, &test_data);
    }

    #[test]
    fn test_print_value_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        DebugTrace::print_value("test", "value");
        DebugTrace::print_value("test", 42);
    }

    #[test]
    fn test_print_value_with_debug_output_disabled() {
        DebugTrace::print_value("test", "value");
        DebugTrace::print_value("test", 42);
    }

    #[test]
    fn test_scope_with_debug_output_enabled() -> Result<(), ErrorTrace> {
        Mediator::new(true).make_current();

        let result: u64 = DebugTrace::scope(
            |scope: &mut DebugTraceScope<'_>| -> Result<u64, ErrorTrace> {
                scope.print_start("test");
                scope.print_field("value", 42);
                scope.print_end();
                Ok(42)
            },
        )?;
        assert_eq!(result, 42);

        let result: u64 = DebugTrace::scope(
            |scope: &mut DebugTraceScope<'_>| -> Result<u64, ErrorTrace> {
                let _ = scope;
                Ok(0)
            },
        )?;
        assert_eq!(result, 0);

        Ok(())
    }

    #[test]
    fn test_scope_with_debug_output_disabled() -> Result<(), ErrorTrace> {
        let result: u64 = DebugTrace::scope(
            |scope: &mut DebugTraceScope<'_>| -> Result<u64, ErrorTrace> {
                assert!(!scope.debug_output);
                scope.print_start("test");
                assert_eq!(scope.output, "");
                Ok(42)
            },
        )?;
        assert_eq!(result, 42);

        Ok(())
    }

    #[test]
    fn test_scope_with_error() -> Result<(), ErrorTrace> {
        Mediator::new(true).make_current();

        let error: ErrorTrace = DebugTrace::scope(
            |scope: &mut DebugTraceScope<'_>| -> Result<u64, ErrorTrace> {
                scope.print("error\n");
                Err(ErrorTrace::new(String::from("test error")))
            },
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "#0 test error");

        Ok(())
    }

    #[test]
    fn test_trace_scope_with_debug_output_enabled() -> Result<(), ErrorTrace> {
        Mediator::new(true).make_current();

        let mediator: Arc<Mediator> = Mediator::current();
        let debug_output: bool = mediator.debug_output;

        let mut scope = DebugTraceScope {
            mediator: &mediator,
            debug_output,
            output: if debug_output {
                String::with_capacity(1024)
            } else {
                String::new()
            },
        };
        assert!(scope.debug_output);
        assert_eq!(scope.output, "");

        scope.print_start("test");
        assert_eq!(scope.output, "test {\n");

        scope.print_field("identifier", "value");
        assert_eq!(scope.output, "test {\n    identifier: value,\n");

        scope.print_field("identifier", 42);
        assert_eq!(
            scope.output,
            "test {\n    identifier: value,\n    identifier: 42,\n"
        );

        scope.print_value("identifier", 42);
        assert_eq!(
            scope.output,
            "test {\n    identifier: value,\n    identifier: 42,\nidentifier: 42\n"
        );

        scope.print("test\n");
        assert_eq!(
            scope.output,
            "test {\n    identifier: value,\n    identifier: 42,\nidentifier: 42\ntest\n"
        );

        let test_data: Vec<u8> = get_test_data();
        let test_data_hexdump: String = format_as_hexdump(&test_data, true);
        let mut expected_string: String = String::from(
            "test {\n    identifier: value,\n    identifier: 42,\nidentifier: 42\ntest\n",
        );

        scope.print_data("test data", 0, &test_data, 16, true);
        expected_string.push_str("test data data of size: 16 at offset: 0 (0x00000000)\n");
        expected_string.push_str(&test_data_hexdump);
        assert_eq!(scope.output, expected_string);

        scope.print_data_field("test data", &test_data);
        expected_string.push_str("    test data\n");
        expected_string.push_str(&test_data_hexdump);
        assert_eq!(scope.output, expected_string);

        let structure_string: String = test_debug_read_data(&test_data);

        scope.print_structure(test_debug_read_data, &test_data);
        expected_string.push_str(&structure_string);
        assert_eq!(scope.output, expected_string);

        scope.print_end();
        expected_string.push_str("}\n\n");
        assert_eq!(scope.output, expected_string);

        Ok(())
    }

    #[test]
    fn test_trace_scope_with_debug_output_disabled() -> Result<(), ErrorTrace> {
        let mediator: Arc<Mediator> = Mediator::current();

        let mut scope = DebugTraceScope {
            mediator: &mediator,
            debug_output: false,
            output: String::new(),
        };
        assert!(!scope.debug_output);

        scope.print("test\n");
        assert_eq!(scope.output, "");

        scope.print_start("test");
        assert_eq!(scope.output, "");

        scope.print_end();
        assert_eq!(scope.output, "");

        scope.print_field("identifier", "value");
        assert_eq!(scope.output, "");

        scope.print_value("identifier", 42);
        assert_eq!(scope.output, "");

        let test_data: Vec<u8> = get_test_data();

        scope.print_data("test data", 0, &test_data, 16, true);
        assert_eq!(scope.output, "");

        scope.print_data_field("test data", &test_data);
        assert_eq!(scope.output, "");

        scope.print_structure(test_debug_read_data, &test_data);
        assert_eq!(scope.output, "");

        Ok(())
    }

    #[test]
    fn test_static_scope_with_debug_output_enabled() {
        Mediator::new(true).make_current();

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            scope.print_start("test");
            scope.print_field("value", 42);
            scope.print_end();
        });
    }

    #[test]
    fn test_static_scope_with_debug_output_disabled() {
        let was_called: Cell<bool> = Cell::new(false);

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            _ = scope;
            was_called.set(true);
        });
        assert!(!was_called.get());
    }

    #[test]
    fn test_static_scope_print() {
        Mediator::new(true).make_current();

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            scope.print("test\n");
            scope.print(String::from("test\n"));
            scope.print(42);
        });
    }

    #[test]
    fn test_static_scope_print_data() {
        Mediator::new(true).make_current();

        let test_data: Vec<u8> = get_test_data();

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            scope.print_data("test data", 0, &test_data, 16, false);
            scope.print_data("test data", 0, &test_data, 16, true);
        });
    }

    #[test]
    fn test_static_scope_print_data_field() {
        Mediator::new(true).make_current();

        let test_data: Vec<u8> = get_test_data();

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            scope.print_data_field("test data", &test_data);
        });
    }

    #[test]
    fn test_static_scope_print_structure() {
        Mediator::new(true).make_current();

        let test_data: Vec<u8> = get_test_data();

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            scope.print_structure(test_debug_read_data, &test_data);
        });
    }

    #[test]
    fn test_static_scope_print_value() {
        Mediator::new(true).make_current();

        DebugTrace::static_scope(|scope: &DebugTraceStaticScope<'_>| {
            scope.print_value("test", "value");
            scope.print_value("test", 42);
        });
    }
}
