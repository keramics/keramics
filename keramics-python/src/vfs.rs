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

use std::io::SeekFrom;
use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use keramics_core::DataStreamReference;
use keramics_vfs::{
    VfsFileEntry, VfsFileSystemReference, VfsFileType, VfsLocation, VfsPath, VfsResolver,
    VfsResolverReference, VfsString, VfsType,
};

use super::datetime::PyDateTime;

#[pyclass]
#[pyo3(name = "VfsDataStream")]
#[derive(Clone)]
struct PyVfsDataStream {
    /// Data steam.
    data_stream: DataStreamReference,
}

#[pymethods]
impl PyVfsDataStream {
    pub fn get_offset(&self) -> PyResult<u64> {
        match self.data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_offset() {
                Ok(offset) => Ok(offset),
                Err(error) => {
                    return Err(PyErr::new::<PyRuntimeError, String>(format!(
                        "Unable to determine offset of data stream with error: {}",
                        error.to_string()
                    )));
                }
            },
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to obtain write lock on data stream with error: {}",
                    error.to_string()
                )));
            }
        }
    }

    pub fn get_size(&self) -> PyResult<u64> {
        match self.data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_size() {
                Ok(size) => Ok(size),
                Err(error) => {
                    return Err(PyErr::new::<PyRuntimeError, String>(format!(
                        "Unable to determine size of data stream with error: {}",
                        error.to_string()
                    )));
                }
            },
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to obtain write lock on data stream with error: {}",
                    error.to_string()
                )));
            }
        }
    }

    #[pyo3(signature = (size=-1), text_signature = "(size=-1, /)")]
    pub fn read(&self, size: i64) -> PyResult<Vec<u8>> {
        // TODO: add support for size == -1 (read all)
        if size < 0 {
            return Err(PyErr::new::<PyRuntimeError, String>(format!(
                "Unsupported size: {}",
                size
            )));
        }
        let mut buffer: Vec<u8> = vec![0; size as usize];

        match self.data_stream.write() {
            Ok(mut data_stream) => match data_stream.read_exact(&mut buffer) {
                Ok(read_count) => read_count,
                Err(error) => {
                    return Err(PyErr::new::<PyRuntimeError, String>(format!(
                        "Unable to read from data stream with error: {}",
                        error.to_string()
                    )));
                }
            },
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to obtain write lock on data stream with error: {}",
                    error.to_string()
                )));
            }
        }
        // TODO: resize buffer if read_count < read_size
        Ok(buffer)
    }

    #[pyo3(signature = (offset, whence=0), text_signature = "(offset, whence=SEEK_SET, /)")]
    pub fn seek(&self, offset: i64, whence: i8) -> PyResult<u64> {
        let position: SeekFrom = match whence {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unsupported whence: {}",
                    whence
                )));
            }
        };
        match self.data_stream.write() {
            Ok(mut data_stream) => match data_stream.seek(position) {
                Ok(offset) => Ok(offset),
                Err(error) => {
                    return Err(PyErr::new::<PyRuntimeError, String>(format!(
                        "Unable to read from data stream with error: {}",
                        error.to_string()
                    )));
                }
            },
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to obtain write lock on data stream with error: {}",
                    error.to_string()
                )));
            }
        }
    }
}

#[pyclass]
#[pyo3(name = "VfsFileEntry")]
#[derive(Clone)]
struct PyVfsFileEntry {
    /// File entry.
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

    // TODO: add deletion time

    #[getter]
    pub fn file_type(&self) -> PyResult<Option<PyVfsFileType>> {
        match self.file_entry.get_file_type() {
            VfsFileType::BlockDevice => Ok(Some(PyVfsFileType::BlockDevice)),
            VfsFileType::CharacterDevice => Ok(Some(PyVfsFileType::CharacterDevice)),
            VfsFileType::Device => Ok(Some(PyVfsFileType::Device)),
            VfsFileType::Directory => Ok(Some(PyVfsFileType::Directory)),
            VfsFileType::File => Ok(Some(PyVfsFileType::File)),
            VfsFileType::NamedPipe => Ok(Some(PyVfsFileType::NamedPipe)),
            VfsFileType::Socket => Ok(Some(PyVfsFileType::Socket)),
            VfsFileType::SymbolicLink => Ok(Some(PyVfsFileType::SymbolicLink)),
            VfsFileType::Unknown(_) => Ok(None),
            VfsFileType::Whiteout => Ok(Some(PyVfsFileType::Whiteout)),
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

    #[getter]
    pub fn size(&self) -> PyResult<u64> {
        Ok(self.file_entry.get_size())
    }

    #[getter]
    pub fn symbolic_link_target(&mut self) -> PyResult<Option<PyVfsPath>> {
        let vfs_file_entry: &mut VfsFileEntry = match Arc::get_mut(&mut self.file_entry) {
            Some(file_entry) => file_entry,
            None => {
                return Err(PyErr::new::<PyRuntimeError, &str>(
                    "Unable to obtain mutable reference to file entry",
                ));
            }
        };
        match vfs_file_entry.get_symbolic_link_target() {
            Ok(Some(link_target)) => Ok(Some(PyVfsPath {
                path: Arc::new(link_target),
            })),
            Ok(None) => Ok(None),
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to retrieve symbolic link target with error: {}",
                    error.to_string()
                )));
            }
        }
    }
}

#[pyclass]
#[pyo3(name = "VfsFileSystem")]
#[derive(Clone)]
struct PyVfsFileSystem {
    /// File system.
    file_system: VfsFileSystemReference,
}

#[pymethods]
impl PyVfsFileSystem {
    pub fn file_entry_exists(&self, path: PyVfsPath) -> PyResult<bool> {
        match self.file_system.file_entry_exists(&path.path) {
            Ok(result) => Ok(result),
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to determine if file entry exists with error: {}",
                    error.to_string()
                )));
            }
        }
    }

    // TODO: add get_data_stream_by_path_and_name

    pub fn get_file_entry_by_path(&self, path: PyVfsPath) -> PyResult<Option<PyVfsFileEntry>> {
        match self.file_system.get_file_entry_by_path(&path.path) {
            Ok(Some(file_entry)) => Ok(Some(PyVfsFileEntry {
                file_entry: Arc::new(file_entry),
            })),
            Ok(None) => {
                return Ok(None);
            }
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to retrieve file entry with error: {}",
                    error.to_string()
                )));
            }
        }
    }

    pub fn get_root_file_entry(&self) -> PyResult<Option<PyVfsFileEntry>> {
        match self.file_system.get_root_file_entry() {
            Ok(Some(file_entry)) => Ok(Some(PyVfsFileEntry {
                file_entry: Arc::new(file_entry),
            })),
            Ok(None) => {
                return Ok(None);
            }
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to retrieve root file entry with error: {}",
                    error.to_string()
                )));
            }
        }
    }
}

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
    /// Location.
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
    /// Resolver.
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

    pub fn get_data_stream_by_location_and_name(
        &self,
        location: PyVfsLocation,
        name: Option<&PyVfsString>,
    ) -> PyResult<Option<PyVfsDataStream>> {
        let vfs_name: Option<&VfsString> = match name {
            Some(name) => Some(&name.string),
            None => None,
        };
        match self
            .resolver
            .get_data_stream_by_location_and_name(&location.location, vfs_name)
        {
            Ok(Some(data_stream)) => Ok(Some(PyVfsDataStream {
                data_stream: data_stream,
            })),
            Ok(None) => {
                return Ok(None);
            }
            Err(error) => {
                return Err(PyErr::new::<PyRuntimeError, String>(format!(
                    "Unable to retrieve data stream with error: {}",
                    error.to_string()
                )));
            }
        }
    }

    pub fn get_file_entry_by_location(
        &self,
        location: PyVfsLocation,
    ) -> PyResult<Option<PyVfsFileEntry>> {
        match self.resolver.get_file_entry_by_location(&location.location) {
            Ok(Some(file_entry)) => Ok(Some(PyVfsFileEntry {
                file_entry: Arc::new(file_entry),
            })),
            Ok(None) => {
                return Ok(None);
            }
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
    /// String.
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
    /// Path.
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
    module.add_class::<PyVfsDataStream>()?;
    module.add_class::<PyVfsFileEntry>()?;
    module.add_class::<PyVfsFileSystem>()?;
    module.add_class::<PyVfsFileType>()?;
    module.add_class::<PyVfsLocation>()?;
    module.add_class::<PyVfsPath>()?;
    module.add_class::<PyVfsResolver>()?;
    module.add_class::<PyVfsString>()?;
    module.add_class::<PyVfsType>()?;

    Ok(())
}
