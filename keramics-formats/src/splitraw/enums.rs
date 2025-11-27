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

/// Segment file naming schema.
#[derive(Debug, PartialEq)]
pub enum SplitRawNamingSchema {
    /// Alphabetic naming schema such as imageaa
    Alphabetic,
    /// Numeric naming schema such as image.1, image.001 or image_001
    Numeric,
    /// "X of N" naming schema such as image.1of5
    XOfN,
}
