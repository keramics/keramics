/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Lock error for value that can be shared across threads.
pub enum SharedValueLockError<T> {
    Poisoned(PoisonError<T>),
    ValueIsNone,
}

impl<T> fmt::Display for SharedValueLockError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            #[cfg(panic = "unwind")]
            SharedValueLockError::Poisoned(ref p) => p.to_string(),
            #[cfg(not(panic = "unwind"))]
            SharedValueLockError::Poisoned(ref p) => match p._never {},
            SharedValueLockError::ValueIsNone => format!("value is none"),
        }
        .fmt(formatter)
    }
}

pub type SharedValueLockResult<Guard> = Result<Guard, SharedValueLockError<Guard>>;

/// Value that can be shared across threads.
pub struct SharedValue<T: ?Sized> {
    value: Option<Arc<RwLock<T>>>,
}

impl<T> SharedValue<T> {
    /// Creates a new value.
    pub fn new(value: T) -> Self {
        Self {
            value: Some(Arc::new(RwLock::new(value))),
        }
    }

    /// Creates a new empty value.
    pub fn none() -> Self {
        Self { value: None }
    }

    /// Clones the value.
    pub fn clone(&self) -> Self {
        let value: Option<Arc<RwLock<T>>> = match &self.value {
            None => None,
            Some(value) => Some(Arc::clone(value)),
        };
        Self { value: value }
    }

    /// Determines if the value is empty.
    pub fn is_none(&self) -> bool {
        self.value.is_none()
    }

    /// Determines if the value is not empty.
    pub fn is_some(&self) -> bool {
        self.value.is_some()
    }

    /// Access the value with a read lock.
    pub fn with_read_lock(&self) -> SharedValueLockResult<RwLockReadGuard<'_, T>> {
        match &self.value {
            None => SharedValueLockResult::Err(SharedValueLockError::ValueIsNone),
            Some(value) => match value.read() {
                Ok(result) => SharedValueLockResult::Ok(result),
                Err(error) => SharedValueLockResult::Err(SharedValueLockError::Poisoned(error)),
            },
        }
    }

    /// Access the value with a write lock.
    pub fn with_write_lock(&self) -> SharedValueLockResult<RwLockWriteGuard<'_, T>> {
        match &self.value {
            None => SharedValueLockResult::Err(SharedValueLockError::ValueIsNone),
            Some(value) => match value.write() {
                Ok(result) => SharedValueLockResult::Ok(result),
                Err(error) => SharedValueLockResult::Err(SharedValueLockError::Poisoned(error)),
            },
        }
    }
}

// TODO: add tests.
