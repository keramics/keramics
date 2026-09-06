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

use keramics_types::Uuid;

use super::enums::BdeKeyProtectorType;

/// BitLocker Drive Encryption (BDE) key protector.
pub struct BdeKeyProtector {
    /// Protector type.
    pub(super) protector_type: BdeKeyProtectorType,

    /// Identifier.
    pub(super) identifier: Uuid,

    /// Offset.
    pub(super) offset: u64,

    /// Size.
    pub(super) size: u16,
}

impl BdeKeyProtector {
    /// Creates a new key protector.
    pub(super) fn new(
        protector_type: BdeKeyProtectorType,
        identifier: Uuid,
        offset: u64,
        size: u16,
    ) -> Self {
        Self {
            protector_type,
            identifier,
            offset,
            size,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Retrieves the protector type.
    pub fn get_protector_type(&self) -> &BdeKeyProtectorType {
        &self.protector_type
    }
}
