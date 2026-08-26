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

/// Debug print data.
#[macro_export]
macro_rules! debug_trace_data {
    ( $description:expr, $offset:expr, $data:expr, $data_size:expr $(,)? ) => {
        #[cfg(feature = "debug-trace")]
        {
            let mediator = $crate::mediator::Mediator::current();

            if mediator.debug_output {
                mediator.debug_print(format!(
                    "{} data of size: {} at offset: {} (0x{:08x})\n",
                    $description, $data_size, $offset, $offset
                ));
                mediator.debug_print_data($data, true);
            }
        }
    };
}

/// Debug print data and a structure representation.
#[macro_export]
macro_rules! debug_trace_data_and_structure {
    ( $description:expr, $offset:expr, $data:expr, $data_size:expr, $structure:expr $(,)? ) => {
        #[cfg(feature = "debug-trace")]
        {
            let mediator = $crate::mediator::Mediator::current();

            if mediator.debug_output {
                mediator.debug_print(format!(
                    "{} data of size: {} at offset: {} (0x{:08x})\n",
                    $description, $data_size, $offset, $offset
                ));
                mediator.debug_print_data($data, true);
                mediator.debug_print($structure);
            }
        }
    };
}

/// Debug print a structure representation.
#[macro_export]
macro_rules! debug_trace_structure {
    ( $structure:expr $(,)? ) => {
        #[cfg(feature = "debug-trace")]
        {
            let mediator = $crate::mediator::Mediator::current();

            if mediator.debug_output {
                mediator.debug_print($structure);
            }
        }
    };
}

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
    ( $message:expr $(,)? ) => {
        $crate::ErrorTrace::new(format!("{}: {}", $crate::error_trace_function!(), $message))
    };
}

/// Creates a new [`ErrorTrace`] based on an existing error.
#[macro_export]
macro_rules! error_trace_new_with_error {
    ( $message:expr, $error:expr $(,)? ) => {
        $crate::ErrorTrace::new(format!(
            "{}: {} with error: {}",
            $crate::error_trace_function!(),
            $message,
            $error.to_string(),
        ))
    };
}

/// Adds a frame to an existing [`ErrorTrace`].
#[macro_export]
macro_rules! error_trace_add_frame {
    ( $error:expr, $message:expr $(,)? ) => {
        $error.add_frame(format!("{}: {}", $crate::error_trace_function!(), $message))
    };
}

/// Retrieves the size of a [`DataStreamReference`].
#[macro_export]
macro_rules! data_stream_get_size {
    ( $data_stream:expr $(,)? ) => {
        match $data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_size() {
                Ok(size) => size,
                Err(mut error) => {
                    $crate::error_trace_add_frame!(
                        error,
                        "Unable to determine size of data stream"
                    );
                    return Err(error);
                }
            },
            Err(error) => {
                return Err($crate::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        }
    };
}

/// Reads data at a specific position from a [`DataStreamReference`].
#[macro_export]
macro_rules! data_stream_read_at_position {
    ( $data_stream:expr, $buf:expr, $pos:expr $(,)? ) => {
        match $data_stream.write() {
            Ok(mut data_stream) => match data_stream.read_at_position($buf, $pos) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    $crate::error_trace_add_frame!(error, "Unable to read from data stream");
                    return Err(error);
                }
            },
            Err(error) => {
                return Err($crate::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        }
    };
}

/// Reads an exact amount of data at a specific position from a [`DataStreamReference`].
#[macro_export]
macro_rules! data_stream_read_exact_at_position {
    ( $data_stream:expr, $buf:expr, $pos:expr $(,)? ) => {
        match $data_stream.write() {
            Ok(mut data_stream) => match data_stream.read_exact_at_position($buf, $pos) {
                Ok(offset) => offset,
                Err(mut error) => {
                    $crate::error_trace_add_frame!(error, "Unable to read from data stream");
                    return Err(error);
                }
            },
            Err(error) => {
                return Err($crate::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::io::SeekFrom;
    use std::sync::mpsc::channel;
    use std::sync::{Arc, RwLock};
    use std::thread::spawn;

    use crate::data_stream::{DataStream, DataStreamReference};
    use crate::errors::ErrorTrace;
    use crate::fake_data_stream::open_fake_data_stream;

    /// Test data stream, where all operations fail.
    struct TestFailingDataStream {}

    impl DataStream for TestFailingDataStream {
        /// Retrieves the current position.
        fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
            Err(ErrorTrace::new(String::from("test error")))
        }

        /// Retrieves the size of the data.
        fn get_size(&mut self) -> Result<u64, ErrorTrace> {
            Err(ErrorTrace::new(String::from("test error")))
        }

        /// Reads data at the current position.
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ErrorTrace> {
            Err(ErrorTrace::new(String::from("test error")))
        }

        /// Sets the current position of the data.
        fn seek(&mut self, _pos: SeekFrom) -> Result<u64, ErrorTrace> {
            Err(ErrorTrace::new(String::from("test error")))
        }
    }

    fn get_test_data() -> Vec<u8> {
        (0..16).collect()
    }

    fn get_test_data_stream() -> DataStreamReference {
        open_fake_data_stream(&get_test_data())
    }

    fn get_test_failing_data_stream() -> DataStreamReference {
        Arc::new(RwLock::new(TestFailingDataStream {}))
    }

    fn get_test_poisoned_data_stream() -> DataStreamReference {
        let data_stream: DataStreamReference = get_test_data_stream();
        let poisoned_data_stream: DataStreamReference = data_stream.clone();
        let (sender, receiver) = channel::<()>();
        let thread = spawn(move || {
            let _guard = poisoned_data_stream
                .write()
                .expect("unable to obtain write lock");

            sender.send(()).expect("unable to send");
            panic!("poison the data stream");
        });
        receiver.recv().expect("unable to receive");
        let _thread_result = thread.join();

        data_stream
    }

    /// Determines the size of a data stream.
    fn data_stream_get_size_helper(data_stream: DataStreamReference) -> Result<u64, ErrorTrace> {
        let size: u64 = data_stream_get_size!(data_stream);

        Ok(size)
    }

    /// Reads data at a position from a data stream.
    fn data_stream_read_at_position_helper(
        data_stream: DataStreamReference,
        buf: &mut [u8],
        pos: SeekFrom,
    ) -> Result<usize, ErrorTrace> {
        let read_count: usize = data_stream_read_at_position!(data_stream, buf, pos);

        Ok(read_count)
    }

    /// Reads an exact amount of data at a position from a data stream.
    fn data_stream_read_exact_at_position_helper(
        data_stream: DataStreamReference,
        buf: &mut [u8],
        pos: SeekFrom,
    ) -> Result<u64, ErrorTrace> {
        let offset: u64 = data_stream_read_exact_at_position!(data_stream, buf, pos);

        Ok(offset)
    }

    #[test]
    fn test_error_trace_function() {
        let function_name: &str = error_trace_function!();

        assert!(function_name.contains("test_error_trace_function"));
    }

    #[test]
    fn test_error_trace_new() {
        let error: ErrorTrace = error_trace_new!("test error");

        let error_message: String = error.to_string();
        assert!(error_message.starts_with("#0 "));
        assert!(error_message.contains("test_error_trace_new: test error"));
    }

    #[test]
    fn test_error_trace_new_with_error() {
        let error: ErrorTrace = error_trace_new_with_error!(
            "test message",
            ErrorTrace::new(String::from("inner error")),
        );

        let error_message: String = error.to_string();
        assert!(error_message.starts_with("#0 "));
        assert!(
            error_message.contains(
                "test_error_trace_new_with_error: test message with error: #0 inner error"
            )
        );
    }

    #[test]
    fn test_error_trace_add_frame() {
        let mut error: ErrorTrace = ErrorTrace::new(String::from("original error"));

        error_trace_add_frame!(error, "additional frame");

        let error_message: String = error.to_string();
        assert!(error_message.contains("#0 original error"));
        assert!(error_message.contains("#1 "));
        assert!(error_message.contains("test_error_trace_add_frame: additional frame"));
    }

    #[test]
    fn test_data_stream_get_size() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_test_data_stream();

        let size: u64 = data_stream_get_size!(data_stream);
        assert_eq!(size, 16);

        Ok(())
    }

    #[test]
    fn test_data_stream_get_size_with_failing_size() {
        let test_data_stream: DataStreamReference = get_test_failing_data_stream();

        let result: Result<u64, ErrorTrace> = data_stream_get_size_helper(test_data_stream);
        let error: ErrorTrace = result.expect_err("expected error");

        let error_message: String = error.to_string();
        assert!(error_message.contains("Unable to determine size of data stream"));
        assert!(error_message.contains("test error"));
    }

    #[test]
    fn test_data_stream_get_size_with_poisoned_data_stream() {
        let test_data_stream: DataStreamReference = get_test_poisoned_data_stream();

        let result: Result<u64, ErrorTrace> = data_stream_get_size_helper(test_data_stream);
        let error: ErrorTrace = result.expect_err("expected error");

        let error_message: String = error.to_string();
        assert!(error_message.contains("Unable to obtain write lock on data stream"));
    }

    #[test]
    fn test_data_stream_read_at_position() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_test_data_stream();

        let mut data: Vec<u8> = vec![0; 4];
        let read_count: usize =
            data_stream_read_at_position!(data_stream, &mut data, SeekFrom::Start(2));

        assert_eq!(read_count, 4);
        assert_eq!(&data, &[2, 3, 4, 5]);

        Ok(())
    }

    #[test]
    fn test_data_stream_read_at_position_beyond_size() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_test_data_stream();

        let mut data: Vec<u8> = vec![0; 24];
        let read_count: usize =
            data_stream_read_at_position!(data_stream, &mut data, SeekFrom::Start(8));

        assert_eq!(read_count, 8);
        assert_eq!(&data[0..8], &[8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(&data[8..24], &[0; 16]);

        Ok(())
    }

    #[test]
    fn test_data_stream_read_at_position_with_failing_read() {
        let test_data_stream: DataStreamReference = get_test_failing_data_stream();

        let mut data: Vec<u8> = vec![0; 4];
        let result: Result<usize, ErrorTrace> =
            data_stream_read_at_position_helper(test_data_stream, &mut data, SeekFrom::Start(0));
        let error: ErrorTrace = result.expect_err("expected error");

        let error_message: String = error.to_string();
        assert!(error_message.contains("Unable to read from data stream"));
        assert!(error_message.contains("test error"));
    }

    #[test]
    fn test_data_stream_read_at_position_with_poisoned_data_stream() {
        let test_data_stream: DataStreamReference = get_test_poisoned_data_stream();

        let mut data: Vec<u8> = vec![0; 4];
        let result: Result<usize, ErrorTrace> =
            data_stream_read_at_position_helper(test_data_stream, &mut data, SeekFrom::Start(0));
        let error: ErrorTrace = result.expect_err("expected error");

        let error_message: String = error.to_string();
        assert!(error_message.contains("Unable to obtain write lock on data stream"));
    }

    #[test]
    fn test_data_stream_read_exact_at_position() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = get_test_data_stream();

        let mut data: Vec<u8> = vec![0; 4];
        let offset: u64 =
            data_stream_read_exact_at_position!(data_stream, &mut data, SeekFrom::Start(2));

        assert_eq!(offset, 2);
        assert_eq!(&data, &[2, 3, 4, 5]);

        Ok(())
    }

    #[test]
    fn test_data_stream_read_exact_at_position_beyond_size() {
        let data_stream: DataStreamReference = get_test_data_stream();

        let mut data: Vec<u8> = vec![0; 24];
        let result: Result<u64, ErrorTrace> =
            data_stream_read_exact_at_position_helper(data_stream, &mut data, SeekFrom::Start(0));
        let error: ErrorTrace = result.expect_err("expected error");

        let error_message: String = error.to_string();
        assert!(error_message.contains("Unable to read the exact amount"));
        assert!(error_message.contains("Unable to read from data stream"));
    }

    #[test]
    fn test_data_stream_read_exact_at_position_with_failing_read() {
        let test_data_stream: DataStreamReference = get_test_failing_data_stream();

        let mut data: Vec<u8> = vec![0; 4];
        let result: Result<u64, ErrorTrace> = data_stream_read_exact_at_position_helper(
            test_data_stream,
            &mut data,
            SeekFrom::Start(0),
        );
        let error: ErrorTrace = result.expect_err("expected error");

        let error_message: String = error.to_string();
        assert!(error_message.contains("Unable to read from data stream"));
        assert!(error_message.contains("test error"));
    }

    #[cfg(feature = "debug-trace")]
    #[test]
    fn test_debug_trace_data() {
        crate::mediator::Mediator::new(true).make_current();

        let data: Vec<u8> = get_test_data();
        debug_trace_data!("test data", 0, &data, 16);
    }

    #[cfg(feature = "debug-trace")]
    #[test]
    fn test_debug_trace_data_with_debug_output_disabled() {
        crate::mediator::Mediator::new(false).make_current();

        let data: Vec<u8> = get_test_data();
        debug_trace_data!("test data", 0, &data, 16);
    }

    #[cfg(feature = "debug-trace")]
    #[test]
    fn test_debug_trace_data_and_structure() {
        crate::mediator::Mediator::new(true).make_current();

        let data: Vec<u8> = get_test_data();
        debug_trace_data_and_structure!("test data", 0, &data, 16, String::from("structure"));
    }

    #[cfg(feature = "debug-trace")]
    #[test]
    fn test_debug_trace_data_and_structure_with_debug_output_disabled() {
        crate::mediator::Mediator::new(false).make_current();

        let data: Vec<u8> = get_test_data();
        debug_trace_data_and_structure!("test data", 0, &data, 16, String::from("structure"));
    }

    #[cfg(feature = "debug-trace")]
    #[test]
    fn test_debug_trace_structure() {
        crate::mediator::Mediator::new(true).make_current();

        debug_trace_structure!(String::from("structure"));
    }

    #[cfg(feature = "debug-trace")]
    #[test]
    fn test_debug_trace_structure_with_debug_output_disabled() {
        crate::mediator::Mediator::new(false).make_current();

        debug_trace_structure!(String::from("structure"));
    }
}
