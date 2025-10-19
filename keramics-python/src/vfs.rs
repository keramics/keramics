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

use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use keramics_vfs::{
    VfsFileEntry, VfsFileSystemReference, VfsFileType, VfsLocation, VfsPath, VfsResolver,
    VfsResolverReference, VfsString, VfsType,
};

use super::datetime::PyDateTime;

#[pyclass]
#[pyo3(name = "VfsFileEntry")]
#[derive(Clone)]
struct PyVfsFileEntry {
    file_entry: Arc<VfsFileEntry>,
}

#[pymethods]
impl PyVfsFileEntry {
    #[getter]
    pub fn access_time(&self) -> PyResult<Option<Py<PyAny>>> {
        match self.file_entry.get_access_time() {
            Some(date_time) => Ok(Some(PyDateTime::new(date_time)?)),
            None => Ok(None),
        }
    }

    #[getter]
    pub fn change_time(&self) -> PyResult<Option<Py<PyAny>>> {
        match self.file_entry.get_change_time() {
            Some(date_time) => Ok(Some(PyDateTime::new(date_time)?)),
            None => Ok(None),
        }
    }

    #[getter]
    pub fn creation_time(&self) -> PyResult<Option<Py<PyAny>>> {
        match self.file_entry.get_creation_time() {
            Some(date_time) => Ok(Some(PyDateTime::new(date_time)?)),
            None => Ok(None),
        }
    }

    #[getter]
    pub fn name(&self) -> PyResult<Option<PyVfsString>> {
        match self.file_entry.get_name() {
            Some(name) => Ok(Some(PyVfsString {
                string: Arc::new(name),
            })),
            None => Ok(None),
        }
    }

    #[getter]
    pub fn modification_time(&self) -> PyResult<Option<Py<PyAny>>> {
        match self.file_entry.get_modification_time() {
            Some(date_time) => Ok(Some(PyDateTime::new(date_time)?)),
            None => Ok(None),
        }
    }
}

#[pyclass]
#[pyo3(name = "VfsFileSystem")]
#[derive(Clone)]
struct PyVfsFileSystem {
    file_system: VfsFileSystemReference,
}

#[pymethods]
impl PyVfsFileSystem {}

#[pyclass(eq)]
#[pyo3(name = "VfsFileType")]
#[derive(Clone, PartialEq)]
pub enum PyVfsFileType {
    #[pyo3(name = "BLOCK_DEVICE")]
    BlockDevice,
    #[pyo3(name = "CHARACTER_DEVICE")]
    CharacterDevice,
    #[pyo3(name = "DEVICE")]
    Device,
    #[pyo3(name = "DIRECTORY")]
    Directory,
    #[pyo3(name = "FILE")]
    File,
    #[pyo3(name = "NAMED_PIPE")]
    NamedPipe,
    #[pyo3(name = "SOCKET")]
    Socket,
    #[pyo3(name = "SYMBOLIC_LINK")]
    SymbolicLink,
    #[pyo3(name = "WHITEOUT")]
    Whiteout,
}

#[pyclass]
#[pyo3(name = "VfsLocation")]
#[derive(Clone)]
struct PyVfsLocation {
    location: Arc<VfsLocation>,
}

#[pymethods]
impl PyVfsLocation {
    #[new]
    #[pyo3(signature = (path_type, path))]
    pub fn new(path_type: PyVfsType, path: PyVfsPath) -> PyResult<Self> {
        let vfs_type: VfsType = match &path_type {
            PyVfsType::Apm => VfsType::Apm,
            PyVfsType::Ext => VfsType::Ext,
            PyVfsType::Ewf => VfsType::Ewf,
            PyVfsType::Fake => VfsType::Fake,
            PyVfsType::Gpt => VfsType::Gpt,
            PyVfsType::Mbr => VfsType::Mbr,
            PyVfsType::Os => VfsType::Os,
            PyVfsType::Qcow => VfsType::Qcow,
            PyVfsType::Vhd => VfsType::Vhd,
            PyVfsType::Vhdx => VfsType::Vhdx,
        };
        let vfs_path: &VfsPath = path.path.as_ref();
        Ok(Self {
            location: Arc::new(VfsLocation::new_base(&vfs_type, vfs_path.clone())),
        })
    }

    pub fn new_with_layer(&self, path_type: PyVfsType, path: PyVfsPath) -> PyResult<Self> {
        let vfs_type: VfsType = match &path_type {
            PyVfsType::Apm => VfsType::Apm,
            PyVfsType::Ext => VfsType::Ext,
            PyVfsType::Ewf => VfsType::Ewf,
            PyVfsType::Fake => VfsType::Fake,
            PyVfsType::Gpt => VfsType::Gpt,
            PyVfsType::Mbr => VfsType::Mbr,
            PyVfsType::Os => VfsType::Os,
            PyVfsType::Qcow => VfsType::Qcow,
            PyVfsType::Vhd => VfsType::Vhd,
            PyVfsType::Vhdx => VfsType::Vhdx,
        };
        let vfs_path: &VfsPath = path.path.as_ref();
        Ok(Self {
            location: Arc::new(self.location.new_with_layer(&vfs_type, vfs_path.clone())),
        })
    }
}

#[pyclass]
#[pyo3(name = "VfsResolver")]
#[derive(Clone)]
struct PyVfsResolver {
    resolver: VfsResolverReference,
}

#[pymethods]
impl PyVfsResolver {
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(Self {
            resolver: VfsResolver::current(),
        })
    }

    pub fn get_file_entry_by_location(
        &self,
        location: PyVfsLocation,
    ) -> PyResult<Option<PyVfsFileEntry>> {
        match self.resolver.get_file_entry_by_location(&location.location) {
            Ok(result) => match result {
                Some(file_entry) => Ok(Some(PyVfsFileEntry {
                    file_entry: Arc::new(file_entry),
                })),
                None => {
                    return Ok(None);
                }
            },
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to retrieve file entry with error: {}",
                    error.to_string()
                )));
            }
        }
    }

    pub fn open_file_system(&self, location: PyVfsLocation) -> PyResult<PyVfsFileSystem> {
        match self.resolver.open_file_system(&location.location) {
            Ok(file_system) => Ok(PyVfsFileSystem {
                file_system: file_system,
            }),
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to open file system with error: {}",
                    error.to_string()
                )));
            }
        }
    }
}

#[pyclass]
#[pyo3(name = "VfsString")]
#[derive(Clone)]
struct PyVfsString {
    string: Arc<VfsString>,
}

#[pymethods]
impl PyVfsString {
    pub fn to_string(&self) -> String {
        self.string.to_string()
    }
}

#[pyclass]
#[pyo3(name = "VfsPath")]
#[derive(Clone)]
struct PyVfsPath {
    path: Arc<VfsPath>,
}

#[pymethods]
impl PyVfsPath {
    #[new]
    #[pyo3(signature = (path_type, path))]
    pub fn new(path_type: PyVfsType, path: &str) -> PyResult<Self> {
        let vfs_type: VfsType = match &path_type {
            PyVfsType::Apm => VfsType::Apm,
            PyVfsType::Ext => VfsType::Ext,
            PyVfsType::Ewf => VfsType::Ewf,
            PyVfsType::Fake => VfsType::Fake,
            PyVfsType::Gpt => VfsType::Gpt,
            PyVfsType::Mbr => VfsType::Mbr,
            PyVfsType::Os => VfsType::Os,
            PyVfsType::Qcow => VfsType::Qcow,
            PyVfsType::Vhd => VfsType::Vhd,
            PyVfsType::Vhdx => VfsType::Vhdx,
        };
        Ok(Self {
            path: Arc::new(VfsPath::from_path(&vfs_type, path)),
        })
    }
}

#[pyclass(eq)]
#[pyo3(name = "VfsType")]
#[derive(Clone, PartialEq)]
pub enum PyVfsType {
    #[pyo3(name = "APM")]
    Apm,
    #[pyo3(name = "EXT")]
    Ext,
    #[pyo3(name = "EWF")]
    Ewf,
    #[pyo3(name = "FAKE")]
    Fake,
    #[pyo3(name = "GPT")]
    Gpt,
    #[pyo3(name = "MBR")]
    Mbr,
    #[pyo3(name = "OS")]
    Os,
    #[pyo3(name = "QCOW")]
    Qcow,
    #[pyo3(name = "VHD")]
    Vhd,
    #[pyo3(name = "VHDX")]
    Vhdx,
}

#[pymodule]
pub fn vfs(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyVfsFileEntry>()?;
    module.add_class::<PyVfsFileSystem>()?;
    module.add_class::<PyVfsLocation>()?;
    module.add_class::<PyVfsPath>()?;
    module.add_class::<PyVfsResolver>()?;
    module.add_class::<PyVfsString>()?;
    module.add_class::<PyVfsType>()?;

    Ok(())
}
