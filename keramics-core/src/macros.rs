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

/// Determines the name of the current function.
#[macro_export]
macro_rules! error_trace_function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

/// Creates a new [`ErrorTrace`].
#[macro_export]
macro_rules! error_trace_new {
    ( $message:expr ) => {
        keramics_core::ErrorTrace::new(format!(
            "{}: {}",
            keramics_core::error_trace_function!(),
            $message,
        ));
    };
}

/// Creates a new [`ErrorTrace`] based on an existing error.
#[macro_export]
macro_rules! error_trace_new_with_error {
    ( $message:expr, $error:expr ) => {
        keramics_core::ErrorTrace::new(format!(
            "{}: {} with error: {}",
            keramics_core::error_trace_function!(),
            $message,
            $error.to_string(),
        ));
    };
}

/// Adds a frame to an existing [`ErrorTrace`].
#[macro_export]
macro_rules! error_trace_add_frame {
    ( $error:expr, $message:expr ) => {
        $error.add_frame(format!(
            "{}: {}",
            keramics_core::error_trace_function!(),
            $message,
        ));
    };
}

/// Retrieves the size of a [`DataStreamReference`].
#[macro_export]
macro_rules! data_stream_get_size {
    ( $data_stream:expr ) => {
        match $data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_size() {
                Ok(size) => size,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to determine size of data stream"
                    );
                    return Err(error);
                }
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
    };
}

/// Reads data at a specific position from a [`DataStreamReference`].
#[macro_export]
macro_rules! data_stream_read_at_position {
    ( $data_stream:expr, $buf:expr, $pos:expr ) => {
        match $data_stream.write() {
            Ok(mut data_stream) => match data_stream.read_at_position($buf, $pos) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read from data stream");
                    return Err(error);
                }
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
    };
}

/// Reads an exact amount of data at a specific position from a [`DataStreamReference`].
#[macro_export]
macro_rules! data_stream_read_exact_at_position {
    ( $data_stream:expr, $buf:expr, $pos:expr ) => {
        match $data_stream.write() {
            Ok(mut data_stream) => match data_stream.read_exact_at_position($buf, $pos) {
                Ok(offset) => offset,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read from data stream");
                    return Err(error);
                }
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
    };
}
