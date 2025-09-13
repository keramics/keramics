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

/// Copy byte values in big-endian order to a 16-bit signed integer.
#[macro_export]
macro_rules! bytes_to_i16_be {
    ( $array:expr, $element:expr ) => {
        i16::from_be_bytes(<[u8; 2]>::try_from(&$array[$element..$element + 2]).unwrap())
    };
}

/// Copy byte values in little-endian order to a 16-bit signed integer.
#[macro_export]
macro_rules! bytes_to_i16_le {
    ( $array:expr, $element:expr ) => {
        i16::from_le_bytes(<[u8; 2]>::try_from(&$array[$element..$element + 2]).unwrap())
    };
}

/// Copy byte values in big-endian order to a 32-bit signed integer.
#[macro_export]
macro_rules! bytes_to_i32_be {
    ( $array:expr, $element:expr ) => {
        i32::from_be_bytes(<[u8; 4]>::try_from(&$array[$element..$element + 4]).unwrap())
    };
}

/// Copy byte values in little-endian order to a 32-bit signed integer.
#[macro_export]
macro_rules! bytes_to_i32_le {
    ( $array:expr, $element:expr ) => {
        i32::from_le_bytes(<[u8; 4]>::try_from(&$array[$element..$element + 4]).unwrap())
    };
}

/// Copy byte values in big-endian order to a 64-bit signed integer.
#[macro_export]
macro_rules! bytes_to_i64_be {
    ( $array:expr, $element:expr ) => {
        i64::from_be_bytes(<[u8; 8]>::try_from(&$array[$element..$element + 8]).unwrap())
    };
}

/// Copy byte values in little-endian order to a 64-bit signed integer.
#[macro_export]
macro_rules! bytes_to_i64_le {
    ( $array:expr, $element:expr ) => {
        i64::from_le_bytes(<[u8; 8]>::try_from(&$array[$element..$element + 8]).unwrap())
    };
}

/// Copy byte values in big-endian order to a 16-bit unsigned integer.
#[macro_export]
macro_rules! bytes_to_u16_be {
    ( $array:expr, $element:expr ) => {
        u16::from_be_bytes(<[u8; 2]>::try_from(&$array[$element..$element + 2]).unwrap())
    };
}

/// Copy byte values in little-endian order to a 16-bit unsigned integer.
#[macro_export]
macro_rules! bytes_to_u16_le {
    ( $array:expr, $element:expr ) => {
        u16::from_le_bytes(<[u8; 2]>::try_from(&$array[$element..$element + 2]).unwrap())
    };
}

/// Copy byte values in big-endian order to a 32-bit unsigned integer.
#[macro_export]
macro_rules! bytes_to_u32_be {
    ( $array:expr, $element:expr ) => {
        u32::from_be_bytes(<[u8; 4]>::try_from(&$array[$element..$element + 4]).unwrap())
    };
}

/// Copy byte values in little-endian order to a 32-bit unsigned integer.
#[macro_export]
macro_rules! bytes_to_u32_le {
    ( $array:expr, $element:expr ) => {
        u32::from_le_bytes(<[u8; 4]>::try_from(&$array[$element..$element + 4]).unwrap())
    };
}

/// Copy byte values in big-endian order to a 64-bit unsigned integer.
#[macro_export]
macro_rules! bytes_to_u64_be {
    ( $array:expr, $element:expr ) => {
        u64::from_be_bytes(<[u8; 8]>::try_from(&$array[$element..$element + 8]).unwrap())
    };
}

/// Copy byte values in little-endian order to a 64-bit unsigned integer.
#[macro_export]
macro_rules! bytes_to_u64_le {
    ( $array:expr, $element:expr ) => {
        u64::from_le_bytes(<[u8; 8]>::try_from(&$array[$element..$element + 8]).unwrap())
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bytes_to_i16_be() {
        let test_data: [u8; 2] = [0x12, 0x34];

        let value_16bit: i16 = bytes_to_i16_be!(test_data, 0);
        assert_eq!(value_16bit, 0x1234);
    }

    #[test]
    fn test_bytes_to_i16_le() {
        let test_data: [u8; 2] = [0x34, 0x12];

        let value_16bit: i16 = bytes_to_i16_le!(test_data, 0);
        assert_eq!(value_16bit, 0x1234);
    }

    #[test]
    fn test_bytes_to_i32_be() {
        let test_data: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

        let value_32bit: i32 = bytes_to_i32_be!(test_data, 0);
        assert_eq!(value_32bit, 0x12345678);
    }

    #[test]
    fn test_bytes_to_i32_le() {
        let test_data: [u8; 4] = [0x78, 0x56, 0x34, 0x12];

        let value_32bit: i32 = bytes_to_i32_le!(test_data, 0);
        assert_eq!(value_32bit, 0x12345678);
    }

    #[test]
    fn test_bytes_to_i64_be() {
        let test_data: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];

        let value_64bit: i64 = bytes_to_i64_be!(test_data, 0);
        assert_eq!(value_64bit, 0x123456789abcdef0);
    }

    #[test]
    fn test_bytes_to_i64_le() {
        let test_data: [u8; 8] = [0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12];

        let value_64bit: i64 = bytes_to_i64_le!(test_data, 0);
        assert_eq!(value_64bit, 0x123456789abcdef0);
    }

    #[test]
    fn test_bytes_to_u16_be() {
        let test_data: [u8; 2] = [0x12, 0x34];

        let value_16bit: u16 = bytes_to_u16_be!(test_data, 0);
        assert_eq!(value_16bit, 0x1234);
    }

    #[test]
    fn test_bytes_to_u16_le() {
        let test_data: [u8; 2] = [0x34, 0x12];

        let value_16bit: u16 = bytes_to_u16_le!(test_data, 0);
        assert_eq!(value_16bit, 0x1234);
    }

    #[test]
    fn test_bytes_to_u32_be() {
        let test_data: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

        let value_32bit: u32 = bytes_to_u32_be!(test_data, 0);
        assert_eq!(value_32bit, 0x12345678);
    }

    #[test]
    fn test_bytes_to_u32_le() {
        let test_data: [u8; 4] = [0x78, 0x56, 0x34, 0x12];

        let value_32bit: u32 = bytes_to_u32_le!(test_data, 0);
        assert_eq!(value_32bit, 0x12345678);
    }

    #[test]
    fn test_bytes_to_u64_be() {
        let test_data: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];

        let value_64bit: u64 = bytes_to_u64_be!(test_data, 0);
        assert_eq!(value_64bit, 0x123456789abcdef0);
    }

    #[test]
    fn test_bytes_to_u64_le() {
        let test_data: [u8; 8] = [0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12];

        let value_64bit: u64 = bytes_to_u64_le!(test_data, 0);
        assert_eq!(value_64bit, 0x123456789abcdef0);
    }
}
