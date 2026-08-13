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

use keramics_core::ErrorTrace;

/// Encryption and decryption context trait.
pub trait CryptContext {
    /// Creates a new context.
    fn new() -> Self
    where
        Self: Sized;

    /// Sets the key.
    fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace>;
}

/// CBC (Cipher Block Chaining) encryption and decryption context trait.
pub trait CryptCbc: CryptContext {
    /// Decrypts data using CBC (Cipher Block Chaining) mode.
    fn decrypt_cbc(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace>;

    /// Encrypts data using CBC (Cipher Block Chaining) mode.
    fn encrypt_cbc(
        &self,
        initialization_vector: &[u8],
        data: &[u8],
        encrypted_data: &mut [u8],
    ) -> Result<(), ErrorTrace>;
}

/// ECB (Electronic CodeBook) encryption and decryption context trait.
pub trait CryptEcb: CryptContext {
    /// Decrypts data using ECB (Electronic CodeBook) mode.
    fn decrypt_ecb(&self, encrypted_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace>;

    /// Encrypts data using ECB (Electronic CodeBook) mode.
    fn encrypt_ecb(&self, data: &[u8], encrypted_data: &mut [u8]) -> Result<(), ErrorTrace>;
}
