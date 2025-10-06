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

use std::error::Error;
use std::fmt;

/// Error with traceback information.
#[derive(Debug)]
pub struct ErrorTrace {
    /// The error messages.
    messages: Vec<String>,
}

impl ErrorTrace {
    /// Creates a new error.
    pub fn new(message_string: String) -> Self {
        Self {
            messages: vec![message_string],
        }
    }

    /// Adds an additional message to the trace.
    pub fn add_frame(&mut self, message_string: String) {
        self.messages.push(message_string);
    }

    /// Retrieves a string representation of the error.
    pub fn to_string(&self) -> String {
        self.messages.join("\n")
    }
}

impl Error for ErrorTrace {}

impl fmt::Display for ErrorTrace {
    /// Formats the error as a string.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            self.messages
                .iter()
                .enumerate()
                .map(|(frame_index, message_string)| format!("#{} {}", frame_index, message_string))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
