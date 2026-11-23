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
use keramics_encryption::{AesContext, AesXtsContext, CryptContext};

use super::cipher_context::BdeCipherContext;
use super::encryption_context::BdeEncryptionContext;
use super::encryption_type::BdeEncryptionType;

/// BitLocker Drive Encryption (BDE) encryption.
pub struct BdeEncryption {}

impl BdeEncryption {
    /// Retrieves a encryption context.
    pub fn get_encryption_context(
        bytes_per_sector: u16,
        encryption_type: &BdeEncryptionType,
        key_data: &[u8],
    ) -> Result<Option<BdeEncryptionContext>, ErrorTrace> {
        let mut cipher_context: BdeCipherContext = match encryption_type.method {
            0x8000..=0x8003 => BdeCipherContext::AesCbc(AesContext::new()),
            0x8004..=0x8005 => BdeCipherContext::AesXts(AesXtsContext::new()),
            _ => return Ok(None),
        };
        let key_size: usize = encryption_type.get_key_size();
        let key_data_size: usize = key_data.len();

        if key_data_size < key_size {
            return Err(keramics_core::error_trace_new!("Unsupported key size"));
        }
        match cipher_context.set_key(&key_data[0..key_size]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in context");
                return Err(error);
            }
        }
        let diffuser_context: Option<AesContext> = match encryption_type.method {
            0x8000 | 0x8001 => {
                if key_data_size < 32 + key_size {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported diffuser key size"
                    ));
                }
                let mut aes_context: AesContext = AesContext::new();

                match aes_context.set_key(&key_data[32..32 + key_size]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to set key in diffuser context"
                        );
                        return Err(error);
                    }
                }
                Some(aes_context)
            }
            _ => None,
        };
        Ok(Some(BdeEncryptionContext::new(
            bytes_per_sector,
            cipher_context,
            diffuser_context,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_encryption_context() -> Result<(), ErrorTrace> {
        let key_data: [u8; 64] = [0x11; 64];

        // Test all supported BDE encryption methods.
        for method in [0x8000, 0x8001, 0x8002, 0x8003, 0x8004, 0x8005] {
            let encryption_type: BdeEncryptionType = BdeEncryptionType::new(method);
            let context: Option<BdeEncryptionContext> =
                BdeEncryption::get_encryption_context(512, &encryption_type, &key_data)?;
            assert!(context.is_some());
        }

        // Test unsupported encryption method returns None.
        let unsupported_type: BdeEncryptionType = BdeEncryptionType::new(0x9999);
        let context: Option<BdeEncryptionContext> =
            BdeEncryption::get_encryption_context(512, &unsupported_type, &key_data)?;
        assert!(context.is_none());

        // Test short key data returns error.
        let cbc_type: BdeEncryptionType = BdeEncryptionType::new(0x8002);
        let result: Result<Option<BdeEncryptionContext>, ErrorTrace> =
            BdeEncryption::get_encryption_context(512, &cbc_type, &[0; 8]);
        assert!(result.is_err());

        // Test short diffuser key data returns error.
        let diffuser_type: BdeEncryptionType = BdeEncryptionType::new(0x8000);
        let result: Result<Option<BdeEncryptionContext>, ErrorTrace> =
            BdeEncryption::get_encryption_context(512, &diffuser_type, &[0; 32]);
        assert!(result.is_err());

        Ok(())
    }
}
