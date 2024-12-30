/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

#![no_main]

use libfuzzer_sys::fuzz_target;

use keramics::hashes::{DigestHashContext, Sha224Context};

/// SHA-224 hash fuzz target.
fuzz_target!(|data: &[u8]| {
    let mut sha224_context: Sha224Context = Sha224Context::new();
    sha224_context.update(&data);
    _ = sha224_context.finalize();
});
