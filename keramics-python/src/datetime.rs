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

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyNone};

use keramics_datetime::{DateTime, Filetime, PosixTime32, PosixTime64Ns};

pub struct PyDateTime {}

impl PyDateTime {
    pub fn new(date_time: &DateTime) -> PyResult<Py<PyAny>> {
        Python::attach(|py| -> PyResult<_> {
            match date_time {
                DateTime::Filetime(filetime) => {
                    let py_filetime: PyFiletime = PyFiletime {
                        filetime: filetime.clone(),
                    };
                    Ok(Py::new(py, py_filetime)?.into_any())
                }
                DateTime::NotSet => {
                    let py_none: Borrowed<'_, '_, PyNone> = PyNone::get(py);
                    Ok(py_none.to_owned().unbind().into_any())
                }
                DateTime::PosixTime32(posix_time) => {
                    let py_posix_time: PyPosixTime32 = PyPosixTime32 {
                        posix_time: posix_time.clone(),
                    };
                    Ok(Py::new(py, py_posix_time)?.into_any())
                }
                DateTime::PosixTime64Ns(posix_time) => {
                    let py_posix_time: PyPosixTime64Ns = PyPosixTime64Ns {
                        posix_time: posix_time.clone(),
                    };
                    Ok(Py::new(py, py_posix_time)?.into_any())
                }
                _ => {
                    todo!();
                }
            }
        })
    }
}

#[pyclass]
#[pyo3(name = "Filetime")]
#[derive(Clone)]
struct PyFiletime {
    filetime: Filetime,
}

#[pymethods]
impl PyFiletime {
    #[getter]
    pub fn timestamp(&self) -> PyResult<u64> {
        Ok(self.filetime.timestamp)
    }
}

#[pyclass]
#[pyo3(name = "PosixTime32")]
#[derive(Clone)]
struct PyPosixTime32 {
    posix_time: PosixTime32,
}

#[pymethods]
impl PyPosixTime32 {
    #[getter]
    pub fn timestamp(&self) -> PyResult<i32> {
        Ok(self.posix_time.timestamp)
    }
}

#[pyclass]
#[pyo3(name = "PosixTime64Ns")]
#[derive(Clone)]
struct PyPosixTime64Ns {
    posix_time: PosixTime64Ns,
}

#[pymethods]
impl PyPosixTime64Ns {
    #[getter]
    pub fn fraction(&self) -> PyResult<u32> {
        Ok(self.posix_time.fraction)
    }

    #[getter]
    pub fn timestamp(&self) -> PyResult<i64> {
        Ok(self.posix_time.timestamp)
    }
}

#[pymodule]
pub fn datetime(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyFiletime>()?;
    module.add_class::<PyPosixTime32>()?;
    module.add_class::<PyPosixTime64Ns>()?;

    Ok(())
}
