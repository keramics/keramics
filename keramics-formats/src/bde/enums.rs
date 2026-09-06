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

use std::fmt;

/// BitLocker disk encryption (BDE) key protector types.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BdeKeyProtectorType {
    ClearKey,
    ExternalKey,
    Passphrase,
    RecoveryPassphrase,
    Tpm,
    TpmAndPin,
    Unknown(u16),
}

impl fmt::Display for BdeKeyProtectorType {
    /// Formats a key protector type for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ClearKey => write!(formatter, "Clear key"),
            Self::ExternalKey => write!(formatter, "External key"),
            Self::Passphrase => write!(formatter, "Passphrase (or password)"),
            Self::RecoveryPassphrase => write!(formatter, "Recovery passphrase (or password)"),
            Self::Tpm => write!(formatter, "TPM"),
            Self::TpmAndPin => write!(formatter, "TPM and pin"),
            Self::Unknown(value) => write!(formatter, "Unknown: 0x{:04x}", value),
        }
    }
}
