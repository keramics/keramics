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
use keramics_encryption::{
    AesContext, AesXtsContext, CryptCbc, CryptContext, CryptEcb, Pbkdf2HmacSha1Context,
    Pbkdf2HmacSha224Context, Pbkdf2HmacSha256Context, Pbkdf2HmacSha512Context,
};
use keramics_hashes::{
    DigestHashContext, Sha1Context, Sha224Context, Sha256Context, Sha512Context,
};

use super::diffuser::LuksDiffuser;
use super::encryption_type::LuksEncryptionType;

/// Linux Unified Key Setup (LUKS) Disk Encryption cipher context.
#[derive(Clone)]
pub enum LuksCipherContext {
    AesCbc(AesContext),
    AesEcb(AesContext),
    AesXts(AesXtsContext),
}

impl LuksCipherContext {
    /// Decrypts data.
    pub fn decrypt(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        match self {
            LuksCipherContext::AesCbc(context) => {
                context.decrypt_cbc(initialization_vector, encrypted_data, data)
            }
            LuksCipherContext::AesEcb(context) => context.decrypt_ecb(encrypted_data, data),
            LuksCipherContext::AesXts(context) => {
                context.decrypt_xts(initialization_vector, encrypted_data, data)
            }
        }
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        match self {
            LuksCipherContext::AesCbc(context) => context.set_key(key),
            LuksCipherContext::AesEcb(context) => context.set_key(key),
            LuksCipherContext::AesXts(context) => {
                let key_size: usize = key.len() / 2;

                context.set_keys(&key[0..key_size], &key[key_size..])
            }
        }
    }
}

/// Linux Unified Key Setup (LUKS) Disk Encryption diffuser context.
pub enum LuksDiffuserContext {
    Sha1(LuksDiffuser<Sha1Context, 20>),
    Sha224(LuksDiffuser<Sha224Context, 28>),
    Sha256(LuksDiffuser<Sha256Context, 32>),
    Sha512(LuksDiffuser<Sha512Context, 64>),
}

impl LuksDiffuserContext {
    /// Merges split key data.
    pub fn merge(&mut self, number_of_stripes: u32, split_data: &[u8], data: &mut [u8]) {
        match self {
            LuksDiffuserContext::Sha1(diffuser) => {
                diffuser.merge(number_of_stripes, split_data, data)
            }
            LuksDiffuserContext::Sha224(diffuser) => {
                diffuser.merge(number_of_stripes, split_data, data)
            }
            LuksDiffuserContext::Sha256(diffuser) => {
                diffuser.merge(number_of_stripes, split_data, data)
            }
            LuksDiffuserContext::Sha512(diffuser) => {
                diffuser.merge(number_of_stripes, split_data, data)
            }
        }
    }
}

/// Linux Unified Key Setup (LUKS) Disk Encryption encryption context.
#[derive(Clone)]
pub struct LuksEncryptionContext {
    /// Cipher context.
    cipher_context: LuksCipherContext,

    /// Initialization vector context.
    intialization_vector_context: LuksInitializationVectorContext,
}

impl LuksEncryptionContext {
    /// Decrypts a sector.
    pub fn decrypt_sector(
        &self,
        sector_number: u64,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut initialization_vector: [u8; 16] = [0; 16];

        match self
            .intialization_vector_context
            .derive_initialization_vector(sector_number, &mut initialization_vector)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to derive initialization vector for sector: {}",
                        sector_number
                    )
                );
                return Err(error);
            }
        }
        match self
            .cipher_context
            .decrypt(&initialization_vector, encrypted_data, data)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt sector: {}", sector_number)
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

/// Linux Unified Key Setup (LUKS) Disk Encryption initialization vector context.
#[derive(Clone)]
pub enum LuksInitializationVectorContext {
    Benbi,
    Essiv(AesContext),
    Null,
    Plain32,
    Plain64,
}

impl LuksInitializationVectorContext {
    /// Derives the initialization vector.
    pub fn derive_initialization_vector(
        &self,
        sector_number: u64,
        initialization_vector: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        match self {
            LuksInitializationVectorContext::Benbi => {
                let block_key: u64 = (sector_number << 5) + 1;

                initialization_vector[0..8].copy_from_slice(&block_key.to_be_bytes());
            }
            LuksInitializationVectorContext::Essiv(crypt_context) => {
                let mut block_key_data: [u8; 16] = [0; 16];
                block_key_data[0..8].copy_from_slice(&sector_number.to_le_bytes());

                // The block key for the initialization vector is encrypted with the hash of the
                // key.
                match crypt_context.encrypt_ecb(&block_key_data, initialization_vector) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to encrypt sector number"
                        );
                        return Err(error);
                    }
                }
            }
            LuksInitializationVectorContext::Null => {}
            LuksInitializationVectorContext::Plain32 => {
                initialization_vector[0..4].copy_from_slice(&(sector_number as u32).to_le_bytes());
            }
            LuksInitializationVectorContext::Plain64 => {
                initialization_vector[0..8].copy_from_slice(&sector_number.to_le_bytes());
            }
        }
        Ok(())
    }
}

/// Linux Unified Key Setup (LUKS) Disk Encryption key derivation context.
pub enum LuksKeyDerivationContext {
    Pbkdf2HmacSha1(Pbkdf2HmacSha1Context),
    Pbkdf2HmacSha224(Pbkdf2HmacSha224Context),
    Pbkdf2HmacSha256(Pbkdf2HmacSha256Context),
    Pbkdf2HmacSha512(Pbkdf2HmacSha512Context),
}

impl LuksKeyDerivationContext {
    /// Derives a key from the password.
    pub fn derive_key(&mut self, password: &[u8], key: &mut [u8]) -> Result<(), ErrorTrace> {
        match self {
            LuksKeyDerivationContext::Pbkdf2HmacSha1(context) => context.derive_key(password, key),
            LuksKeyDerivationContext::Pbkdf2HmacSha224(context) => {
                context.derive_key(password, key)
            }
            LuksKeyDerivationContext::Pbkdf2HmacSha256(context) => {
                context.derive_key(password, key)
            }
            LuksKeyDerivationContext::Pbkdf2HmacSha512(context) => {
                context.derive_key(password, key)
            }
        }
    }
}

/// Linux Unified Key Setup (LUKS) Disk Encryption encryption.
pub struct LuksEncryption {}

impl LuksEncryption {
    /// Retrieves a digest hash context.
    fn get_digest_hash_context(hashing_method: &str) -> Option<Box<dyn DigestHashContext>> {
        // TODO: ripemd160
        // TODO: wd256
        match hashing_method {
            "sha1" => Some(Box::new(Sha1Context::new())),
            "sha224" => Some(Box::new(Sha224Context::new())),
            "sha256" => Some(Box::new(Sha256Context::new())),
            "sha512" => Some(Box::new(Sha512Context::new())),
            _ => None,
        }
    }

    /// Retrieves a diffuser context.
    pub fn get_diffuser_context(hashing_method: &str) -> Option<LuksDiffuserContext> {
        // TODO: ripemd160
        // TODO: wd256
        match hashing_method {
            "sha1" => Some(LuksDiffuserContext::Sha1(
                LuksDiffuser::<Sha1Context, 20>::new(),
            )),
            "sha224" => Some(LuksDiffuserContext::Sha224(
                LuksDiffuser::<Sha224Context, 28>::new(),
            )),
            "sha256" => Some(LuksDiffuserContext::Sha256(
                LuksDiffuser::<Sha256Context, 32>::new(),
            )),
            "sha512" => Some(LuksDiffuserContext::Sha512(
                LuksDiffuser::<Sha512Context, 64>::new(),
            )),
            _ => None,
        }
    }

    /// Retrieves an encryption context.
    pub fn get_encryption_context(
        encryption_type: &LuksEncryptionType,
        key: &[u8],
    ) -> Result<Option<LuksEncryptionContext>, ErrorTrace> {
        // TODO: arc4: cbc, ecb
        // TODO: anubis
        // TODO: blowfish: cbc, ecb
        // TODO: cast5
        // TODO: cast6
        // TODO: serpent: cbc, ebc
        // TODO: tnepres
        // TODO: twofish
        let mut cipher_context: LuksCipherContext = match encryption_type.encryption_method.as_str()
        {
            "aes" => match encryption_type.chaining_mode.as_str() {
                "cbc" => LuksCipherContext::AesCbc(AesContext::new()),
                "ecb" => LuksCipherContext::AesEcb(AesContext::new()),
                "xts" => LuksCipherContext::AesXts(AesXtsContext::new()),
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        match cipher_context.set_key(key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in context");
                return Err(error);
            }
        }
        let mut intialization_vector_context: LuksInitializationVectorContext =
            match Self::get_initialization_vector_context(encryption_type, key) {
                Ok(Some(context)) => context,
                Ok(None) => return Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to create initialization vector context"
                    );
                    return Err(error);
                }
            };
        Ok(Some(LuksEncryptionContext {
            cipher_context,
            intialization_vector_context,
        }))
    }

    /// Retrieves an initialization vector context.
    fn get_initialization_vector_context(
        encryption_type: &LuksEncryptionType,
        key: &[u8],
    ) -> Result<Option<LuksInitializationVectorContext>, ErrorTrace> {
        // TODO: lmk
        // TODO: plumb
        match encryption_type.initialization_vector_mode.as_deref() {
            Some("benbi") => {
                if encryption_type.initialization_vector_options.is_some() {
                    return Ok(None);
                }
                Ok(Some(LuksInitializationVectorContext::Benbi))
            }
            Some("essiv") => {
                if encryption_type.encryption_method.as_str() != "aes"
                    || (encryption_type.chaining_mode.as_str() != "cbc"
                        && encryption_type.chaining_mode.as_str() != "ecb")
                {
                    return Ok(None);
                }
                let mut digest_context: Box<dyn DigestHashContext> = match &encryption_type
                    .initialization_vector_options
                {
                    Some(hashing_method) => match Self::get_digest_hash_context(hashing_method) {
                        Some(context) => context,
                        None => return Ok(None),
                    },
                    None => return Ok(None),
                };
                digest_context.update(key);

                let digest_hash: Vec<u8> = digest_context.finalize();
                let digest_hash_size: usize = digest_hash.len();

                let mut essiv_key: [u8; 32] = [0; 32];
                essiv_key[0..digest_hash_size].copy_from_slice(&digest_hash);

                let mut essiv_context: AesContext = AesContext::new();

                match essiv_context.set_key(&essiv_key) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to set key in ESSIV context"
                        );
                        return Err(error);
                    }
                }
                Ok(Some(LuksInitializationVectorContext::Essiv(essiv_context)))
            }
            Some("null") => {
                if encryption_type.initialization_vector_options.is_some() {
                    return Ok(None);
                }
                Ok(Some(LuksInitializationVectorContext::Null))
            }
            Some("plain") => {
                if encryption_type.initialization_vector_options.is_some() {
                    return Ok(None);
                }
                Ok(Some(LuksInitializationVectorContext::Plain32))
            }
            Some("plain64") => {
                if encryption_type.initialization_vector_options.is_some() {
                    return Ok(None);
                }
                Ok(Some(LuksInitializationVectorContext::Plain64))
            }
            None => Ok(Some(LuksInitializationVectorContext::Null)),
            _ => Ok(None),
        }
    }

    /// Retrieves a key derivation context.
    pub fn get_key_derivation_context(
        hashing_method: &str,
        salt: &[u8],
        number_of_iterations: usize,
    ) -> Result<Option<LuksKeyDerivationContext>, ErrorTrace> {
        // TODO: ripemd160
        // TODO: wd256
        match hashing_method {
            "sha1" => Ok(Some(LuksKeyDerivationContext::Pbkdf2HmacSha1(
                Pbkdf2HmacSha1Context::new(salt, number_of_iterations),
            ))),
            "sha224" => Ok(Some(LuksKeyDerivationContext::Pbkdf2HmacSha224(
                Pbkdf2HmacSha224Context::new(salt, number_of_iterations),
            ))),
            "sha256" => Ok(Some(LuksKeyDerivationContext::Pbkdf2HmacSha256(
                Pbkdf2HmacSha256Context::new(salt, number_of_iterations),
            ))),
            "sha512" => Ok(Some(LuksKeyDerivationContext::Pbkdf2HmacSha512(
                Pbkdf2HmacSha512Context::new(salt, number_of_iterations),
            ))),
            _ => Ok(None),
        }
    }
}
