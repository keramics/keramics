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

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple digest hash context for testing default trait methods.
    struct TestDigestHashContext {
        /// Stored bytes.
        bytes: Vec<u8>,
    }

    impl DigestHashContext for TestDigestHashContext {
        /// Creates a new context.
        fn new() -> Self {
            Self { bytes: Vec::new() }
        }

        /// Finalizes the digest hash calculation.
        fn finalize(&mut self) -> Vec<u8> {
            self.bytes.clone()
        }

        /// Calculates the digest hash of the data.
        fn update(&mut self, data: &[u8]) {
            self.bytes.extend_from_slice(data);
        }
    }

    #[test]
    fn test_finalize_into() {
        let mut context: TestDigestHashContext = TestDigestHashContext::new();
        context.update(b"test");

        let mut output: [u8; 4] = [0; 4];
        context.finalize_into(&mut output);
        assert_eq!(output, *b"test");
    }
}
