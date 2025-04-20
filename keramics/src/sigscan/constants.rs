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

/// Lookup table to map a common byte values.
/// TODO: consider using a frequency table instead.
pub const SIGSCAN_COMMON_BYTE_VALUES: [bool; 256] = [
    true,  // 0x00
    true,  // 0x01
    false, // 0x02
    false, // 0x03
    false, // 0x04
    false, // 0x05
    false, // 0x06
    false, // 0x07 '\a'
    false, // 0x08 '\b'
    true,  // 0x09 '\t'
    true,  // 0x0a '\n'
    false, // 0x0b '\v'
    false, // 0x0c '\f'
    true,  // 0x0d '\r'
    false, // 0x0e
    false, // 0x0f
    false, // 0x10
    false, // 0x11
    false, // 0x12
    false, // 0x13
    false, // 0x14
    false, // 0x15
    false, // 0x16
    false, // 0x17
    false, // 0x18
    false, // 0x19
    false, // 0x1a
    false, // 0x1b
    false, // 0x1c
    false, // 0x1d
    false, // 0x1e
    false, // 0x1f
    true,  // 0x20 ' '
    false, // 0x21 '!'
    false, // 0x22
    false, // 0x23
    false, // 0x24
    false, // 0x25
    false, // 0x26
    false, // 0x27
    false, // 0x28
    false, // 0x29
    false, // 0x2a
    false, // 0x2b
    false, // 0x2c
    false, // 0x2d
    false, // 0x2e
    false, // 0x2f
    true,  // 0x30 '0'
    true,  // 0x31 '1'
    true,  // 0x32 '2'
    true,  // 0x33 '3'
    true,  // 0x34 '4'
    true,  // 0x35 '5'
    true,  // 0x36 '6'
    true,  // 0x37 '7'
    true,  // 0x38 '8'
    true,  // 0x39 '9'
    false, // 0x3a ':'
    false, // 0x3b ';'
    false, // 0x3c '<'
    false, // 0x3d '='
    false, // 0x3e '>'
    false, // 0x3f '?'
    false, // 0x40 '@'
    true,  // 0x41 'A'
    true,  // 0x42 'B'
    true,  // 0x43 'C'
    true,  // 0x44 'D'
    true,  // 0x45 'E'
    true,  // 0x46 'F'
    true,  // 0x47 'G'
    true,  // 0x48 'H'
    true,  // 0x49 'I'
    true,  // 0x4a 'J'
    true,  // 0x4b 'K'
    true,  // 0x4c 'L'
    true,  // 0x4d 'M'
    true,  // 0x4e 'N'
    true,  // 0x4f 'O'
    true,  // 0x50 'P'
    true,  // 0x51 'Q'
    true,  // 0x52 'R'
    true,  // 0x53 'S'
    true,  // 0x54 'T'
    true,  // 0x55 'U'
    true,  // 0x56 'V'
    true,  // 0x57 'W'
    true,  // 0x58 'X'
    true,  // 0x59 'Y'
    true,  // 0x5a 'Z'
    false, // 0x5b
    false, // 0x5c
    false, // 0x5d
    false, // 0x5e
    false, // 0x5f
    false, // 0x60
    true,  // 0x61 'a'
    true,  // 0x62 'b'
    true,  // 0x63 'c'
    true,  // 0x64 'd'
    true,  // 0x65 'e'
    true,  // 0x66 'f'
    true,  // 0x67 'g'
    true,  // 0x68 'h'
    true,  // 0x69 'i'
    true,  // 0x6a 'j'
    true,  // 0x6b 'k'
    true,  // 0x6c 'l'
    true,  // 0x6d 'm'
    true,  // 0x6e 'n'
    true,  // 0x6f 'o'
    true,  // 0x70 'p'
    true,  // 0x71 'q'
    true,  // 0x72 'r'
    true,  // 0x73 's'
    true,  // 0x74 't'
    true,  // 0x75 'u'
    true,  // 0x76 'v'
    true,  // 0x77 'w'
    true,  // 0x78 'x'
    true,  // 0x79 'y'
    true,  // 0x7a 'z'
    false, // 0x7b '{'
    false, // 0x7c '|'
    false, // 0x7d '}'
    false, // 0x7e '~'
    false, // 0x7f
    false, // 0x80
    false, // 0x81
    false, // 0x82
    false, // 0x83
    false, // 0x84
    false, // 0x85
    false, // 0x86
    false, // 0x87
    false, // 0x88
    false, // 0x89
    false, // 0x8a
    false, // 0x8b
    false, // 0x8c
    false, // 0x8d
    false, // 0x8e
    false, // 0x8f
    false, // 0x90
    false, // 0x91
    false, // 0x92
    false, // 0x93
    false, // 0x94
    false, // 0x95
    false, // 0x96
    false, // 0x97
    false, // 0x98
    false, // 0x99
    false, // 0x9a
    false, // 0x9b
    false, // 0x9c
    false, // 0x9d
    false, // 0x9e
    false, // 0x9f
    false, // 0xa0
    false, // 0xa1
    false, // 0xa2
    false, // 0xa3
    false, // 0xa4
    false, // 0xa5
    false, // 0xa6
    false, // 0xa7
    false, // 0xa8
    false, // 0xa9
    false, // 0xaa
    false, // 0xab
    false, // 0xac
    false, // 0xad
    false, // 0xae
    false, // 0xaf
    false, // 0xb0
    false, // 0xb1
    false, // 0xb2
    false, // 0xb3
    false, // 0xb4
    false, // 0xb5
    false, // 0xb6
    false, // 0xb7
    false, // 0xb8
    false, // 0xb9
    false, // 0xba
    false, // 0xbb
    false, // 0xbc
    false, // 0xbd
    false, // 0xbe
    false, // 0xbf
    false, // 0xc0
    false, // 0xc1
    false, // 0xc2
    false, // 0xc3
    false, // 0xc4
    false, // 0xc5
    false, // 0xc6
    false, // 0xc7
    false, // 0xc8
    false, // 0xc9
    false, // 0xca
    false, // 0xcb
    false, // 0xcc
    false, // 0xcd
    false, // 0xce
    false, // 0xcf
    false, // 0xd0
    false, // 0xd1
    false, // 0xd2
    false, // 0xd3
    false, // 0xd4
    false, // 0xd5
    false, // 0xd6
    false, // 0xd7
    false, // 0xd8
    false, // 0xd9
    false, // 0xda
    false, // 0xdb
    false, // 0xdc
    false, // 0xdd
    false, // 0xde
    false, // 0xdf
    false, // 0xe0
    false, // 0xe1
    false, // 0xe2
    false, // 0xe3
    false, // 0xe4
    false, // 0xe5
    false, // 0xe6
    false, // 0xe7
    false, // 0xe8
    false, // 0xe9
    false, // 0xea
    false, // 0xeb
    false, // 0xec
    false, // 0xed
    false, // 0xee
    false, // 0xef
    false, // 0xf0
    false, // 0xf1
    false, // 0xf2
    false, // 0xf3
    false, // 0xf4
    false, // 0xf5
    false, // 0xf6
    false, // 0xf7
    false, // 0xf8
    false, // 0xf9
    false, // 0xfa
    false, // 0xfb
    false, // 0xfc
    false, // 0xfd
    false, // 0xfe
    true,  // 0xff
];
