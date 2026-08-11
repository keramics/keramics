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

/// Universal Disk Image Format (UDIF) credential types.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UdifCredentialType {
    Certificate,
    Passphrase,
}

/// Universal Disk Image Format (UDIF) credential.
#[derive(Clone, Debug)]
pub struct UdifCredential {
    /// Credential type.
    pub(super) credential_type: UdifCredentialType,

    /// Data.
    pub(super) data: Vec<u8>,
}

impl UdifCredential {
    /// Creates a new credential.
    pub fn new(credential_type: UdifCredentialType, data: &[u8]) -> Self {
        Self {
            credential_type,
            data: data.to_vec(),
        }
    }
}
