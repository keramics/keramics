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

use crate::block_stream::BlockStream;

use super::block_reader::EwfBlockReader;

/// Expert Witness Compression Format (EWF) block stream.
pub type EwfBlockStream = BlockStream<EwfBlockReader>;

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;
    use std::path::PathBuf;

    use keramics_core::{DataStream, ErrorTrace};

    use crate::ewf::block_range::{EwfBlockRange, EwfBlockRangeType};
    use crate::ewf::enums::EwfNamingSchema;
    use crate::file_resolver::FileResolverReference;
    use crate::os_file_resolver::open_os_file_resolver;
    use crate::tests::get_test_data_path;

    fn get_block_stream() -> Result<EwfBlockStream, ErrorTrace> {
        let path_string: String = get_test_data_path("ewf");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        let block_ranges: [EwfBlockRange; 128] = [
            EwfBlockRange {
                media_offset: 0,
                segment_number: 1,
                data_offset: 1945,
                data_size: 721,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 32768,
                segment_number: 1,
                data_offset: 2666,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 65536,
                segment_number: 1,
                data_offset: 2718,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 98304,
                segment_number: 1,
                data_offset: 2770,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 131072,
                segment_number: 1,
                data_offset: 2822,
                data_size: 283,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 163840,
                segment_number: 1,
                data_offset: 3105,
                data_size: 450,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 196608,
                segment_number: 1,
                data_offset: 3555,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 229376,
                segment_number: 1,
                data_offset: 3607,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 262144,
                segment_number: 1,
                data_offset: 3659,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 294912,
                segment_number: 1,
                data_offset: 3711,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 327680,
                segment_number: 1,
                data_offset: 3763,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 360448,
                segment_number: 1,
                data_offset: 3815,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 393216,
                segment_number: 1,
                data_offset: 3867,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 425984,
                segment_number: 1,
                data_offset: 3919,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 458752,
                segment_number: 1,
                data_offset: 3971,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 491520,
                segment_number: 1,
                data_offset: 4023,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 524288,
                segment_number: 1,
                data_offset: 4075,
                data_size: 71,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 557056,
                segment_number: 1,
                data_offset: 4146,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 589824,
                segment_number: 1,
                data_offset: 4198,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 622592,
                segment_number: 1,
                data_offset: 4250,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 655360,
                segment_number: 1,
                data_offset: 4302,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 688128,
                segment_number: 1,
                data_offset: 4354,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 720896,
                segment_number: 1,
                data_offset: 4406,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 753664,
                segment_number: 1,
                data_offset: 4458,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 786432,
                segment_number: 1,
                data_offset: 4510,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 819200,
                segment_number: 1,
                data_offset: 4562,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 851968,
                segment_number: 1,
                data_offset: 4614,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 884736,
                segment_number: 1,
                data_offset: 4666,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 917504,
                segment_number: 1,
                data_offset: 4718,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 950272,
                segment_number: 1,
                data_offset: 4770,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 983040,
                segment_number: 1,
                data_offset: 4822,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1015808,
                segment_number: 1,
                data_offset: 4874,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1048576,
                segment_number: 1,
                data_offset: 4926,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1081344,
                segment_number: 1,
                data_offset: 4978,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1114112,
                segment_number: 1,
                data_offset: 5030,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1146880,
                segment_number: 1,
                data_offset: 5082,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1179648,
                segment_number: 1,
                data_offset: 5134,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1212416,
                segment_number: 1,
                data_offset: 5186,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1245184,
                segment_number: 1,
                data_offset: 5238,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1277952,
                segment_number: 1,
                data_offset: 5290,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1310720,
                segment_number: 1,
                data_offset: 5342,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1343488,
                segment_number: 1,
                data_offset: 5394,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1376256,
                segment_number: 1,
                data_offset: 5446,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1409024,
                segment_number: 1,
                data_offset: 5498,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1441792,
                segment_number: 1,
                data_offset: 5550,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1474560,
                segment_number: 1,
                data_offset: 5602,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1507328,
                segment_number: 1,
                data_offset: 5654,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1540096,
                segment_number: 1,
                data_offset: 5706,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1572864,
                segment_number: 1,
                data_offset: 5758,
                data_size: 91,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1605632,
                segment_number: 1,
                data_offset: 5849,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1638400,
                segment_number: 1,
                data_offset: 5901,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1671168,
                segment_number: 1,
                data_offset: 5953,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1703936,
                segment_number: 1,
                data_offset: 6005,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1736704,
                segment_number: 1,
                data_offset: 6057,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1769472,
                segment_number: 1,
                data_offset: 6109,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1802240,
                segment_number: 1,
                data_offset: 6161,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1835008,
                segment_number: 1,
                data_offset: 6213,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1867776,
                segment_number: 1,
                data_offset: 6265,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1900544,
                segment_number: 1,
                data_offset: 6317,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1933312,
                segment_number: 1,
                data_offset: 6369,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1966080,
                segment_number: 1,
                data_offset: 6421,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 1998848,
                segment_number: 1,
                data_offset: 6473,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2031616,
                segment_number: 1,
                data_offset: 6525,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2064384,
                segment_number: 1,
                data_offset: 6577,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2097152,
                segment_number: 1,
                data_offset: 6629,
                data_size: 92,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2129920,
                segment_number: 1,
                data_offset: 6721,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2162688,
                segment_number: 1,
                data_offset: 6773,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2195456,
                segment_number: 1,
                data_offset: 6825,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2228224,
                segment_number: 1,
                data_offset: 6877,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2260992,
                segment_number: 1,
                data_offset: 6929,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2293760,
                segment_number: 1,
                data_offset: 6981,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2326528,
                segment_number: 1,
                data_offset: 7033,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2359296,
                segment_number: 1,
                data_offset: 7085,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2392064,
                segment_number: 1,
                data_offset: 7137,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2424832,
                segment_number: 1,
                data_offset: 7189,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2457600,
                segment_number: 1,
                data_offset: 7241,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2490368,
                segment_number: 1,
                data_offset: 7293,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2523136,
                segment_number: 1,
                data_offset: 7345,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2555904,
                segment_number: 1,
                data_offset: 7397,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2588672,
                segment_number: 1,
                data_offset: 7449,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2621440,
                segment_number: 1,
                data_offset: 7501,
                data_size: 92,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2654208,
                segment_number: 1,
                data_offset: 7593,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2686976,
                segment_number: 1,
                data_offset: 7645,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2719744,
                segment_number: 1,
                data_offset: 7697,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2752512,
                segment_number: 1,
                data_offset: 7749,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2785280,
                segment_number: 1,
                data_offset: 7801,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2818048,
                segment_number: 1,
                data_offset: 7853,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2850816,
                segment_number: 1,
                data_offset: 7905,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2883584,
                segment_number: 1,
                data_offset: 7957,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2916352,
                segment_number: 1,
                data_offset: 8009,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2949120,
                segment_number: 1,
                data_offset: 8061,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 2981888,
                segment_number: 1,
                data_offset: 8113,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3014656,
                segment_number: 1,
                data_offset: 8165,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3047424,
                segment_number: 1,
                data_offset: 8217,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3080192,
                segment_number: 1,
                data_offset: 8269,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3112960,
                segment_number: 1,
                data_offset: 8321,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3145728,
                segment_number: 1,
                data_offset: 8373,
                data_size: 4096,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3178496,
                segment_number: 1,
                data_offset: 12469,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3211264,
                segment_number: 1,
                data_offset: 12521,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3244032,
                segment_number: 1,
                data_offset: 12573,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3276800,
                segment_number: 1,
                data_offset: 12625,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3309568,
                segment_number: 1,
                data_offset: 12677,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3342336,
                segment_number: 1,
                data_offset: 12729,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3375104,
                segment_number: 1,
                data_offset: 12781,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3407872,
                segment_number: 1,
                data_offset: 12833,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3440640,
                segment_number: 1,
                data_offset: 12885,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3473408,
                segment_number: 1,
                data_offset: 12937,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3506176,
                segment_number: 1,
                data_offset: 12989,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3538944,
                segment_number: 1,
                data_offset: 13041,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3571712,
                segment_number: 1,
                data_offset: 13093,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3604480,
                segment_number: 1,
                data_offset: 13145,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3637248,
                segment_number: 1,
                data_offset: 13197,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3670016,
                segment_number: 1,
                data_offset: 13249,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3702784,
                segment_number: 1,
                data_offset: 13301,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3735552,
                segment_number: 1,
                data_offset: 13353,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3768320,
                segment_number: 1,
                data_offset: 13405,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3801088,
                segment_number: 1,
                data_offset: 13457,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3833856,
                segment_number: 1,
                data_offset: 13509,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3866624,
                segment_number: 1,
                data_offset: 13561,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3899392,
                segment_number: 1,
                data_offset: 13613,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3932160,
                segment_number: 1,
                data_offset: 13665,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3964928,
                segment_number: 1,
                data_offset: 13717,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 3997696,
                segment_number: 1,
                data_offset: 13769,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 4030464,
                segment_number: 1,
                data_offset: 13821,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 4063232,
                segment_number: 1,
                data_offset: 13873,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 4096000,
                segment_number: 1,
                data_offset: 13925,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 4128768,
                segment_number: 1,
                data_offset: 13977,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
            EwfBlockRange {
                media_offset: 4161536,
                segment_number: 1,
                data_offset: 14029,
                data_size: 52,
                range_type: EwfBlockRangeType::Compressed,
            },
        ];
        Ok(EwfBlockStream::new(EwfBlockReader::new(
            &file_resolver,
            "ext2",
            Some(EwfNamingSchema::E01UpperCase).as_ref(),
            32768,
            &block_ranges,
            4194304,
        )))
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;

        block_stream.seek(SeekFrom::Start(1024))?;

        let offset: u64 = block_stream.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;

        let size: u64 = block_stream.get_size()?;
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;
        let size: u64 = block_stream.get_size()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;

        let offset = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = block_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;

        let result: Result<u64, ErrorTrace> = block_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;
        let size: u64 = block_stream.get_size()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;
        block_stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x04, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x43, 0x0f,
            0x00, 0x00, 0xe3, 0x03, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x0a, 0xea, 0x78, 0x67, 0x0a, 0xea, 0x78, 0x67, 0x02, 0x00, 0xff, 0xff,
            0x53, 0xef, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x09, 0xea, 0x78, 0x67, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0b, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x57, 0x1e, 0x25, 0x97, 0x42, 0xa1, 0x4d, 0x6a,
            0xad, 0xa9, 0xcd, 0xb1, 0x19, 0x1b, 0x5d, 0xea, 0x65, 0x78, 0x74, 0x32, 0x5f, 0x74,
            0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2f, 0x6d, 0x6e, 0x74,
            0x2f, 0x6b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, 0x43,
            0x11, 0xae, 0xbe, 0xdb, 0x40, 0x41, 0xa4, 0xb6, 0xf5, 0x6b, 0x15, 0x34, 0xd6, 0x66,
            0x01, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0xea,
            0x78, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: EwfBlockStream = get_block_stream()?;
        block_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
