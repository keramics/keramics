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

mod data_stream;
mod enums;
mod errors;
mod fake_data_stream;
pub mod formatters;
pub mod macros;
pub mod mediator;
mod os_data_stream;

pub use data_stream::{DataStream, DataStreamReference};
pub use enums::ByteOrder;
pub use errors::ErrorTrace;
pub use fake_data_stream::{FakeDataStream, open_fake_data_stream};
pub use os_data_stream::open_os_data_stream;
