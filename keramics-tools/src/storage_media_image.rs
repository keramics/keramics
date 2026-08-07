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

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::ewf::EwfImage;
use keramics_formats::pdi::{PdiImage, PdiImageLayer};
use keramics_formats::qcow::{QcowImage, QcowImageLayer};
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_formats::sparseimage::SparseImageFile;
use keramics_formats::splitraw::SplitRawImage;
use keramics_formats::udif::UdifImage;
use keramics_formats::vhd::{VhdImage, VhdImageLayer};
use keramics_formats::vhdx::{VhdxImage, VhdxImageLayer};
use keramics_formats::vmdk::{VmdkImage, VmdkImageLayer};
use keramics_formats::{
    FileResolverReference, FormatIdentifier, FormatScanner, PathComponent, open_os_file_resolver,
};

/// Storage media image.
pub enum StorageMediaImage {
    Ewf {
        ewf_image: Arc<RwLock<EwfImage>>,
    },
    Pdi {
        pdi_image_layer: Arc<RwLock<PdiImageLayer>>,
    },
    Qcow {
        qcow_image_layer: QcowImageLayer,
    },
    SparseBundle {
        sparsebundle_image: Arc<RwLock<SparseBundleImage>>,
    },
    SparseImage {
        sparseimage_file: Arc<RwLock<SparseImageFile>>,
    },
    SplitRaw {
        splitraw_image: Arc<RwLock<SplitRawImage>>,
    },
    Udif {
        udif_image: Arc<RwLock<UdifImage>>,
    },
    Vhd {
        vhd_image_layer: VhdImageLayer,
    },
    Vhdx {
        vhdx_image_layer: VhdxImageLayer,
    },
    Vmdk {
        vmdk_image_layer: Arc<RwLock<VmdkImageLayer>>,
    },
}

impl StorageMediaImage {
    /// Opens a storage media image.
    fn get_base_path_and_file_name<'a>(
        path: &'a PathBuf,
    ) -> Result<(PathBuf, &'a str), ErrorTrace> {
        let mut base_path: PathBuf = path.clone();
        base_path.pop();

        let file_name: &str = match path.file_name() {
            Some(file_name_path) => match file_name_path.to_str() {
                Some(path_string) => path_string,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported source - invalid file name"
                    ));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported source - missing file name"
                ));
            }
        };
        Ok((base_path, file_name))
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        match self {
            Self::Ewf { ewf_image } => ewf_image.clone(),
            Self::Pdi {
                pdi_image_layer, ..
            } => pdi_image_layer.clone(),
            Self::Qcow {
                qcow_image_layer, ..
            } => qcow_image_layer.clone(),
            Self::SparseBundle { sparsebundle_image } => sparsebundle_image.clone(),
            Self::SparseImage { sparseimage_file } => sparseimage_file.clone(),
            Self::SplitRaw { splitraw_image } => splitraw_image.clone(),
            Self::Udif { udif_image } => udif_image.clone(),
            Self::Vhd {
                vhd_image_layer, ..
            } => vhd_image_layer.clone(),
            Self::Vhdx {
                vhdx_image_layer, ..
            } => vhdx_image_layer.clone(),
            Self::Vmdk {
                vmdk_image_layer, ..
            } => vmdk_image_layer.clone(),
        }
    }

    /// Retrieves the stored MD5 hash.
    pub fn get_md5_hash(&self) -> Result<Option<Vec<u8>>, ErrorTrace> {
        match self {
            Self::Ewf { ewf_image } => match ewf_image.read() {
                Ok(image) => Ok(Some(image.md5_hash.to_vec())),
                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on EWF image",
                    error
                )),
            },
            _ => Ok(None),
        }
    }

    /// Retrieves the stored SHA1 hash.
    pub fn get_sha1_hash(&self) -> Result<Option<Vec<u8>>, ErrorTrace> {
        match self {
            Self::Ewf { ewf_image } => match ewf_image.read() {
                Ok(image) => Ok(Some(image.sha1_hash.to_vec())),
                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on EWF image",
                    error
                )),
            },
            _ => Ok(None),
        }
    }

    /// Opens a storage media image.
    pub fn open(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        if path.is_dir() && path.extension() == Some("sparsebundle".as_ref()) {
            match Self::open_sparsebundle_image(path) {
                Ok(storage_media_image) => return Ok(storage_media_image),
                Err(_) => {
                    return Err(keramics_core::error_trace_new!(
                        "No known storage media image formats found"
                    ));
                }
            }
        }
        let data_stream: DataStreamReference = match open_os_data_stream(path) {
            Ok(data_stream) => data_stream,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
                return Err(error);
            }
        };
        match Self::scan_for_storage_image_formats(&data_stream) {
            Ok(Some(format_identifier)) => match format_identifier {
                FormatIdentifier::Ewf => Self::open_ewf_image(path),
                FormatIdentifier::Pdi => Self::open_pdi_image(path),
                FormatIdentifier::Qcow => Self::open_qcow_image(path),
                FormatIdentifier::SparseImage => Self::open_sparseimage_file(path),
                FormatIdentifier::Udif => Self::open_udif_image(path),
                FormatIdentifier::Vhd => Self::open_vhd_image(path),
                FormatIdentifier::Vhdx => Self::open_vhdx_image(path),
                FormatIdentifier::Vmdk => Self::open_vmdk_image(path),
                _ => Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format: {}",
                    format_identifier.to_string()
                ))),
            },
            Ok(None) => {
                match Self::open_splitraw_image(path) {
                    Ok(storage_media_image) => Ok(storage_media_image),
                    Err(_) => {
                        // TODO: scan for known volume and file system formats to detect raw
                        // storage media image format.
                        Err(keramics_core::error_trace_new!(
                            "No known storage media image formats found"
                        ))
                    }
                }
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for known storage media image format signatures"
                );
                Err(error)
            }
        }
    }

    /// Opens an EWF image.
    fn open_ewf_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut ewf_image: EwfImage = EwfImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match ewf_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                return Err(error);
            }
        }
        Ok(Self::Ewf {
            ewf_image: Arc::new(RwLock::new(ewf_image)),
        })
    }

    /// Opens a PDI image.
    fn open_pdi_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, _) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for base path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut pdi_image: PdiImage = PdiImage::new();

        match pdi_image.open(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open PDI image");
                return Err(error);
            }
        }
        let number_of_layers: usize = pdi_image.get_number_of_layers();

        match pdi_image.get_layer_by_index(number_of_layers - 1) {
            Ok(pdi_image_layer) => Ok(Self::Pdi { pdi_image_layer }),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve top image layer");
                Err(error)
            }
        }
    }

    /// Opens a QCOW image.
    fn open_qcow_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for base path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut qcow_image: QcowImage = QcowImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match qcow_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
                return Err(error);
            }
        }
        let number_of_layers: usize = qcow_image.get_number_of_layers();

        match qcow_image.get_layer_by_index(number_of_layers - 1) {
            Ok(qcow_image_layer) => Ok(Self::Qcow { qcow_image_layer }),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve top image layer");
                Err(error)
            }
        }
    }

    /// Opens a sparsebundle image.
    fn open_sparsebundle_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let file_resolver: FileResolverReference = match open_os_file_resolver(&path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut sparsebundle_image: SparseBundleImage = SparseBundleImage::new();

        match sparsebundle_image.open(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open sparsebundle image");
                return Err(error);
            }
        }
        Ok(Self::SparseBundle {
            sparsebundle_image: Arc::new(RwLock::new(sparsebundle_image)),
        })
    }

    /// Opens a sparseimage file.
    fn open_sparseimage_file(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let data_stream: DataStreamReference = match open_os_data_stream(path) {
            Ok(data_stream) => data_stream,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open data stream: {}", path.display())
                );
                return Err(error);
            }
        };
        let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

        match sparseimage_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read sparseimage file from data stream"
                );
                return Err(error);
            }
        }
        Ok(Self::SparseImage {
            sparseimage_file: Arc::new(RwLock::new(sparseimage_file)),
        })
    }

    /// Opens a split raw image.
    fn open_splitraw_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut splitraw_image: SplitRawImage = SplitRawImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match splitraw_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open split raw image");
                return Err(error);
            }
        }
        Ok(Self::SplitRaw {
            splitraw_image: Arc::new(RwLock::new(splitraw_image)),
        })
    }

    /// Opens an UDIF image.
    fn open_udif_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut udif_image: UdifImage = UdifImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match udif_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open UDIF image");
                return Err(error);
            }
        }
        Ok(Self::Udif {
            udif_image: Arc::new(RwLock::new(udif_image)),
        })
    }

    /// Opens a VHD image.
    fn open_vhd_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for base path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut vhd_image: VhdImage = VhdImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match vhd_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHD image");
                return Err(error);
            }
        }
        let number_of_layers: usize = vhd_image.get_number_of_layers();

        match vhd_image.get_layer_by_index(number_of_layers - 1) {
            Ok(vhd_image_layer) => Ok(Self::Vhd { vhd_image_layer }),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve top VHD image layer"
                );
                Err(error)
            }
        }
    }

    /// Opens a VHDX image.
    fn open_vhdx_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for base path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut vhdx_image: VhdxImage = VhdxImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match vhdx_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                return Err(error);
            }
        }
        let number_of_layers: usize = vhdx_image.get_number_of_layers();

        match vhdx_image.get_layer_by_index(number_of_layers - 1) {
            Ok(vhdx_image_layer) => Ok(Self::Vhdx { vhdx_image_layer }),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve top image layer");
                Err(error)
            }
        }
    }

    /// Opens a VMDK image.
    fn open_vmdk_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
            Ok(result) => result,
            Err(mut error) => {
                // TODO: get printable version of path instead of using display().
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine base path and file name of path: {}",
                        path.display()
                    )
                );
                return Err(error);
            }
        };
        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to create file resolver for path: {}",
                        base_path.display()
                    )
                );
                return Err(error);
            }
        };
        let mut vmdk_image: VmdkImage = VmdkImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match vmdk_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
                return Err(error);
            }
        }
        let number_of_layers: usize = vmdk_image.get_number_of_layers();

        match vmdk_image.get_layer_by_index(number_of_layers - 1) {
            Ok(vmdk_image_layer) => Ok(Self::Vmdk { vmdk_image_layer }),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve top image layer");
                Err(error)
            }
        }
    }

    /// Scans a data stream for storage media image format signatures.
    fn scan_for_storage_image_formats(
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        let mut format_scanner: FormatScanner = FormatScanner::new();
        format_scanner.add_ewf_signatures();
        format_scanner.add_pdi_signatures();
        format_scanner.add_qcow_signatures();
        // TODO: support for sparse bundle.
        format_scanner.add_sparseimage_signatures();
        format_scanner.add_udif_signatures();
        format_scanner.add_vhd_signatures();
        format_scanner.add_vhdx_signatures();
        format_scanner.add_vmdk_signatures();

        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let mut scan_results: HashSet<FormatIdentifier> =
            match format_scanner.scan_data_stream(data_stream) {
                Ok(scan_results) => scan_results,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to scan data stream for known format signatures"
                    );
                    return Err(error);
                }
            };
        if scan_results.len() > 1 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported multiple known format signatures"
            ));
        }
        let result: Option<FormatIdentifier> = scan_results.drain().next();

        Ok(result)
    }
}
