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

use super::fat::{FatDate, FatTimeDate, FatTimeDate10Ms};
use super::filetime::Filetime;
use super::posix::{PosixTime32, PosixTime64Ns};

#[derive(Clone, Debug, PartialEq)]
pub enum DateTime {
    FatDate(FatDate),
    FatTimeDate(FatTimeDate),
    FatTimeDate10Ms(FatTimeDate10Ms),
    Filetime(Filetime),
    HfsTime,
    NotSet,
    PosixTime32(PosixTime32),
    PosixTime64Ns(PosixTime64Ns),
}
