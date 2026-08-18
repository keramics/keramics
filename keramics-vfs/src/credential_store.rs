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

use std::sync::{OnceLock, RwLock};
use std::vec::IntoIter;

use keramics_core::ErrorTrace;

use super::credential::VfsCredential;

/// Virtual File System (VFS) credential store.
pub struct VfsCredentialStore {
    /// Credentials.
    credentials: RwLock<Vec<VfsCredential>>,
}

impl VfsCredentialStore {
    /// Retrieves the credential store.
    pub fn current() -> &'static Self {
        static INSTANCE: OnceLock<VfsCredentialStore> = OnceLock::new();

        INSTANCE.get_or_init(|| Self {
            credentials: RwLock::new(Vec::new()),
        })
    }

    /// Adds a passphrase.
    pub fn add_passphrase(&self, password: &[u8]) -> Result<(), ErrorTrace> {
        match self.credentials.write() {
            Ok(mut credentials) => {
                credentials.push(VfsCredential::Passphrase(password.to_vec()));
                Ok(())
            }
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to obtain write lock on credentials",
                error
            )),
        }
    }

    /// Retrieves a credentials iterator.
    pub fn iter(&self) -> IntoIter<VfsCredential> {
        let credentials: Vec<VfsCredential> = match self.credentials.read() {
            Ok(credentials) => credentials.clone(),
            Err(_) => Vec::new(),
        };
        credentials.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_passphrase() -> Result<(), ErrorTrace> {
        let credential_store: &VfsCredentialStore = VfsCredentialStore::current();

        assert_eq!(credential_store.iter().count(), 0);

        credential_store.add_passphrase("KeRaMiCs".as_bytes())?;

        assert_eq!(credential_store.iter().count(), 1);

        Ok(())
    }
}
