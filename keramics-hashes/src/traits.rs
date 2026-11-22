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

/// Digest hash context trait.
pub trait DigestHashContext {
    /// Creates a new context.
    fn new() -> Self
    where
        Self: Sized;

    /// Finalizes the digest hash calculation.
    fn finalize(&mut self) -> Vec<u8>;

    /// Finalizes the digest hash calculation into a fixed-size output buffer.
    fn finalize_into(&mut self, output: &mut [u8]) {
        let hash: Vec<u8> = self.finalize();
        output[0..hash.len()].copy_from_slice(&hash);
    }

    /// Calculates the digest hash of the data.
    fn update(&mut self, data: &[u8]);
}
