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
use keramics_formats::{Path, PathCharacterMappings, PathComponent};

/// Path filter signature.
#[derive(Debug, PartialEq)]
pub struct PathFilterSignature {
    /// Path.
    pub(super) path: Path,

    /// Data fork name.
    pub(super) data_fork_name: Option<PathComponent>,
}

impl PathFilterSignature {
    /// Create a new path filter signature.
    pub fn new(path: Path, data_fork_name: Option<PathComponent>) -> Self {
        Self {
            path,
            data_fork_name,
        }
    }

    /// Creates a new path filter signature with case folding applied.
    pub fn new_with_case_folding(
        &self,
        mappings: &PathCharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        let path: Path = match self.path.new_with_case_folding(mappings) {
            Ok(case_folded_path) => case_folded_path,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to apply case folding on path"
                );
                return Err(error);
            }
        };
        let data_fork_name: Option<PathComponent> = match &self.data_fork_name {
            Some(name) => match name.new_with_case_folding(mappings) {
                Ok(case_folded_name) => Some(case_folded_name),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to apply case folding on data fork name"
                    );
                    return Err(error);
                }
            },
            None => None,
        };
        Ok(Self {
            path,
            data_fork_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use keramics_types::constants::UCS2_CASE_MAPPINGS;
    use keramics_types::{Ucs2CharacterMappings, Ucs2String};

    #[test]
    fn test_new_with_case_folding() -> Result<(), ErrorTrace> {
        let mappings: PathCharacterMappings = PathCharacterMappings::Ucs2(Arc::new(
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice()),
        ));

        let signature: PathFilterSignature = PathFilterSignature::new(
            Path::from("/Library/Caches"),
            Some(PathComponent::from("rsrc")),
        );
        let folded_signature: PathFilterSignature = signature.new_with_case_folding(&mappings)?;
        assert_eq!(
            folded_signature.path,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::Ucs2String(Ucs2String::from("LIBRARY")),
                    PathComponent::Ucs2String(Ucs2String::from("CACHES")),
                ]
            }
        );
        assert_eq!(
            folded_signature.data_fork_name,
            Some(PathComponent::Ucs2String(Ucs2String::from("RSRC")))
        );

        // Test without a data fork name.
        let signature: PathFilterSignature =
            PathFilterSignature::new(Path::from("/Library/Caches"), None);
        let folded_signature: PathFilterSignature = signature.new_with_case_folding(&mappings)?;
        assert_eq!(folded_signature.data_fork_name, None);

        // Test with a path outside the UCS-2 range.
        let invalid_path_signature: PathFilterSignature =
            PathFilterSignature::new(Path::from("/\u{1f600}"), None);
        let result: Result<PathFilterSignature, ErrorTrace> =
            invalid_path_signature.new_with_case_folding(&mappings);
        assert!(result.is_err());

        // Test with a data fork name outside the UCS-2 range.
        let invalid_fork_signature: PathFilterSignature = PathFilterSignature::new(
            Path::from("/Library/Caches"),
            Some(PathComponent::from("\u{1f600}")),
        );
        let result: Result<PathFilterSignature, ErrorTrace> =
            invalid_fork_signature.new_with_case_folding(&mappings);
        assert!(result.is_err());

        Ok(())
    }
}
