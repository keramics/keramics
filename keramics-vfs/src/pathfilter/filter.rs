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

use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_formats::{Path, PathCharacterMappings, PathComponent};

use super::enums::ScanTreeType;
use super::scan_tree::ScanTree;
use super::signature::PathFilterSignature;

/// Path filter.
pub struct PathFilter {
    /// Scan paths.
    signatures: Vec<Arc<PathFilterSignature>>,

    /// Case folding mappings; `None` means exact match.
    case_folding_mappings: Option<PathCharacterMappings>,

    /// Prefix scan tree.
    prefix_scan_tree: ScanTree,

    /// Suffix scan tree.
    suffix_scan_tree: ScanTree,
}

impl PathFilter {
    /// Creates a new path filter.
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            case_folding_mappings: None,
            prefix_scan_tree: ScanTree::new(ScanTreeType::Prefix),
            suffix_scan_tree: ScanTree::new(ScanTreeType::Suffix),
        }
    }

    /// Sets the case folding mappings used to normalize signature paths, component keys,
    /// and query paths. Must be called before `add_signature` and `build`.
    pub fn set_case_folding(&mut self, case_folding_mappings: PathCharacterMappings) {
        self.case_folding_mappings = Some(case_folding_mappings);
    }

    /// Adds a signature. If case folding is enabled, the path and data fork name are
    /// normalized to their folded form before being stored.
    pub fn add_signature(&mut self, signature: PathFilterSignature) -> Result<(), ErrorTrace> {
        let signature: PathFilterSignature = match &self.case_folding_mappings {
            Some(mappings) => PathFilterSignature {
                path: signature.path.new_with_case_folding(mappings)?,
                data_fork_name: match signature.data_fork_name {
                    Some(name) => Some(name.new_with_case_folding(mappings)?),
                    None => None,
                },
            },
            None => signature,
        };
        self.signatures.push(Arc::new(signature));

        Ok(())
    }

    /// Builds the scan trees.
    pub fn build(&mut self) -> Result<(), ErrorTrace> {
        match self.prefix_scan_tree.build(&self.signatures) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to build prefix scan tree");
                return Err(error);
            }
        }
        // TODO: add support for suffix scan tree

        Ok(())
    }

    /// Determines whether the given path matches the filter. If case folding is enabled,
    /// `path` and `data_fork_name` are normalized before comparison.
    pub fn is_match(
        &self,
        path: &Path,
        data_fork_name: Option<&PathComponent>,
    ) -> Result<bool, ErrorTrace> {
        let (path, data_fork_name): (Path, Option<PathComponent>) =
            match &self.case_folding_mappings {
                Some(mappings) => {
                    let path: Path = path.new_with_case_folding(mappings)?;
                    let data_fork_name: Option<PathComponent> = match data_fork_name {
                        Some(name) => Some(name.new_with_case_folding(mappings)?),
                        None => None,
                    };
                    (path, data_fork_name)
                }
                None => (path.clone(), data_fork_name.cloned()),
            };

        match self.prefix_scan_tree.scan_path(&path) {
            Some(signature) => Ok(data_fork_name.as_ref() == signature.data_fork_name.as_ref()),
            None => {
                // TODO: add support for suffix scan tree
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::Ucs2CharacterMappings;

    #[test]
    fn test_add_signature() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        assert_eq!(path_filter.signatures.len(), 0);

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ))?;
        assert_eq!(path_filter.signatures.len(), 1);
        assert_eq!(
            path_filter.signatures[0].path,
            Path::from("/Windows/System32/winevt/Logs/Application.evtx")
        );
        assert_eq!(path_filter.signatures[0].data_fork_name, None);

        Ok(())
    }

    #[test]
    fn test_add_signature_with_data_fork_name() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        let data_fork_name: PathComponent = PathComponent::from("rsrc");
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Library/Caches/com.apple.Safari"),
            Some(data_fork_name.clone()),
        ))?;
        assert_eq!(path_filter.signatures.len(), 1);
        assert_eq!(
            path_filter.signatures[0].path,
            Path::from("/Library/Caches/com.apple.Safari")
        );
        assert_eq!(
            path_filter.signatures[0].data_fork_name,
            Some(data_fork_name)
        );

        Ok(())
    }

    #[test]
    fn test_add_multiple_signatures() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ))?;
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb"),
            None,
        ))?;
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Library/Caches/com.apple.Safari"),
            Some(PathComponent::from("rsrc")),
        ))?;

        assert_eq!(path_filter.signatures.len(), 3);

        Ok(())
    }

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ))?;
        path_filter.build()?;

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Application.evtx");
        assert!(path_filter.is_match(&path, None)?);

        Ok(())
    }

    #[test]
    fn test_build_without_signatures() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.build()?;

        // Without signatures no path matches the filter.
        let path: Path = Path::from("/Windows/System32/winevt/Logs/Application.evtx");
        assert!(!path_filter.is_match(&path, None)?);

        Ok(())
    }

    #[test]
    fn test_is_match() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ))?;
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb"),
            None,
        ))?;
        path_filter.build()?;

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Application.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, true);

        let path: Path = Path::from("/Windows/SoftwareDistribution/DataStore/DataStore.edb");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, true);

        let path: Path = Path::from("/Windows/System32/winevt/Logs/Security.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        Ok(())
    }

    fn get_ascii_case_folding_mappings() -> PathCharacterMappings {
        // ASCII upper-case to lower-case mappings, sufficient for path tests below.
        let mut mappings: Ucs2CharacterMappings = Ucs2CharacterMappings::new();
        for letter in b'A'..=b'Z' {
            mappings.add(letter as u16, (letter + 0x20) as u16);
        }
        PathCharacterMappings::Ucs2(Arc::new(mappings))
    }

    #[test]
    fn test_is_match_with_data_fork_name() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Library/Caches/com.apple.Safari"),
            Some(PathComponent::from("rsrc")),
        ))?;
        path_filter.build()?;

        let path: Path = Path::from("/Library/Caches/com.apple.Safari");

        // The data fork name must match the signature.
        let data_fork_name: PathComponent = PathComponent::from("rsrc");
        let result: bool = path_filter.is_match(&path, Some(&data_fork_name))?;
        assert_eq!(result, true);

        let data_fork_name: PathComponent = PathComponent::from("data");
        let result: bool = path_filter.is_match(&path, Some(&data_fork_name))?;
        assert_eq!(result, false);

        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_is_match_with_case_folding() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();
        path_filter.set_case_folding(get_ascii_case_folding_mappings());

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Windows/System32/winevt/Logs/Application.evtx"),
            None,
        ))?;
        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/Library/Caches/com.apple.Safari"),
            Some(PathComponent::from("rsrc")),
        ))?;
        path_filter.build()?;

        // Mixed-case query matches the case-folded signature.
        let path: Path = Path::from("/windows/system32/winevt/logs/application.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, true);

        // Case folding must not cause a false positive on a different path.
        let path: Path = Path::from("/windows/system32/winevt/logs/security.evtx");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        // Upper-case query matching a lower-case signature with a data fork name.
        let path: Path = Path::from("/LIBRARY/CACHES/com.apple.Safari");
        let data_fork_name: PathComponent = PathComponent::from("RSRC");
        let result: bool = path_filter.is_match(&path, Some(&data_fork_name))?;
        assert_eq!(result, true);

        Ok(())
    }

    #[test]
    fn test_is_match_without_case_folding_is_case_sensitive() -> Result<(), ErrorTrace> {
        let mut path_filter: PathFilter = PathFilter::new();

        path_filter.add_signature(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        ))?;
        path_filter.build()?;

        let path: Path = Path::from("/TESTDIR1/testfile1");
        let result: bool = path_filter.is_match(&path, None)?;
        assert_eq!(result, false);

        Ok(())
    }
}
