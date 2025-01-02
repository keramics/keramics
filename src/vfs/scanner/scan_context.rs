/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use super::scan_node::VfsScanNode;
use super::traits::VfsScannerMediator;

/// Virtual File System (VFS) scan context.
pub struct VfsScanContext<'a> {
    /// Mediator.
    mediator: Option<&'a dyn VfsScannerMediator>,

    /// Root node.
    pub root_node: Option<VfsScanNode>,
}

impl<'a> VfsScanContext<'a> {
    /// Creates a new scan context.
    pub fn new() -> Self {
        Self {
            mediator: None,
            root_node: None,
        }
    }

    /// Sets a mediator.
    pub fn set_mediator(&mut self, mediator: &'a dyn VfsScannerMediator) {
        self.mediator = Some(mediator);
    }
}
