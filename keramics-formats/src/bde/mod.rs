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

mod aes_ccm_encrypted_key;
mod block_range;
mod block_reader;
mod block_stream;
mod boot_record;
mod boot_record_descriptor;
mod boot_record_togo;
mod boot_record_vista;
pub mod constants;
mod credential;
mod encrypted_volume;
mod encryption;
mod encryption_context;
mod encryption_type;
mod enums;
mod key_protector;
mod metadata_block;
mod metadata_block_header;
mod metadata_entry_header;
mod metadata_header;
mod metadata_property;
mod password;
mod stretch_key;
mod volume_master_key;

pub use credential::BdeCredential;
pub use encrypted_volume::BdeEncryptedVolume;
pub use encryption_type::BdeEncryptionType;
pub use enums::BdeKeyProtectorType;
pub use key_protector::BdeKeyProtector;
