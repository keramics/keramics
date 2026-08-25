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
use std::sync::{Arc, RwLock};

use super::formatters::format_as_hexdump;

pub type MediatorReference = Arc<Mediator>;

/// Mediator.
pub struct Mediator {
    /// Debug output.
    pub debug_output: bool,
}

impl Mediator {
    /// Creates a new mediator.
    pub fn new(debug_output: bool) -> Self {
        Self { debug_output }
    }

    /// Retrieves the current mediator.
    #[cfg_attr(feature = "no-inline", inline(never))]
    pub fn current() -> Arc<Mediator> {
        CURRENT_MEDIATOR.with(|mediator| mediator.read().unwrap().clone())
    }

    /// Creates the current mediator.
    pub fn make_current(self) {
        CURRENT_MEDIATOR.with(|mediator| *mediator.write().unwrap() = Arc::new(self))
    }

    /// Prints a string for debugging.
    pub fn debug_print<T: Display>(&self, text: T) {
        if self.debug_output {
            print!("{}", text);
        }
    }

    /// Prints data for debugging.
    pub fn debug_print_data(&self, data: &[u8], group: bool) {
        if self.debug_output {
            print!("{}", format_as_hexdump(data, group));
        }
    }
}

thread_local! {
    static CURRENT_MEDIATOR: RwLock<Arc<Mediator>> = RwLock::new(Arc::new(Mediator::new(false)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_print() {
        let mediator: Arc<Mediator> = Mediator::current();

        mediator.debug_print("test");
    }

    #[test]
    fn test_debug_print_enabled() {
        let mediator: Arc<Mediator> = Arc::new(Mediator::new(true));

        mediator.debug_print("test");
    }

    #[test]
    fn test_debug_print_data() {
        let mediator: Arc<Mediator> = Mediator::current();

        let data: Vec<u8> = (0..16).collect();
        mediator.debug_print_data(&data, false);
    }

    #[test]
    fn test_debug_print_data_grouped() {
        let mediator: Arc<Mediator> = Arc::new(Mediator::new(true));

        let data: Vec<u8> = (0..32).collect();
        mediator.debug_print_data(&data, true);
    }

    #[test]
    fn test_make_current() {
        let original_debug_output: bool = Mediator::current().debug_output;

        let mediator: Mediator = Mediator::new(true);
        mediator.make_current();

        let current: Arc<Mediator> = Mediator::current();
        assert!(current.debug_output);

        let mediator: Mediator = Mediator::new(original_debug_output);
        mediator.make_current();

        let current: Arc<Mediator> = Mediator::current();
        assert_eq!(current.debug_output, original_debug_output);
    }
}
