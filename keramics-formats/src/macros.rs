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

/// Decompresses DEFLATE compressed data.
#[macro_export]
macro_rules! deflate_decompress {
    ($compressed:expr, $uncompressed:expr, $error_message:expr) => {{
        #[cfg(feature = "zlib")]
        use zlib_rs::{InflateConfig, ReturnCode, decompress_slice};

        #[cfg(not(feature = "zlib"))]
        use keramics_compression::DeflateContext;

        #[cfg(feature = "zlib")]
        {
            let mut inflate_config: InflateConfig = InflateConfig::default();
            inflate_config.window_bits = -15;

            let (decompressed_slice, return_code) =
                decompress_slice($uncompressed, $compressed, inflate_config);
            if return_code != ReturnCode::Ok {
                return Err(keramics_core::error_trace_new!($error_message));
            }
            decompressed_slice.len()
        }
        #[cfg(not(feature = "zlib"))]
        {
            let mut deflate_context: DeflateContext = DeflateContext::new();

            match deflate_context.decompress($compressed, $uncompressed) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, $error_message);
                    return Err(error);
                }
            }
            deflate_context.uncompressed_data_size
        }
    }};
}

/// Decompresses zlib compressed data.
#[macro_export]
macro_rules! zlib_decompress {
    ($compressed:expr, $uncompressed:expr, $error_message:expr) => {{
        #[cfg(feature = "zlib")]
        use zlib_rs::{InflateConfig, ReturnCode, decompress_slice};

        #[cfg(not(feature = "zlib"))]
        use keramics_compression::ZlibContext;

        #[cfg(feature = "zlib")]
        {
            let inflate_config: InflateConfig = InflateConfig::default();

            let (decompressed_slice, return_code) =
                decompress_slice($uncompressed, $compressed, inflate_config);
            if return_code != ReturnCode::Ok {
                return Err(keramics_core::error_trace_new!($error_message));
            }
            decompressed_slice.len()
        }
        #[cfg(not(feature = "zlib"))]
        {
            let mut zlib_context: ZlibContext = ZlibContext::new();

            match zlib_context.decompress($compressed, $uncompressed) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, $error_message);
                    return Err(error);
                }
            }
            zlib_context.uncompressed_data_size
        }
    }};
}
