# Apple Data Compression (ADC) data format

ADC compression is used in various data formats used on Mac OS, including
[Universal Disk Image Format (UDIF)](udif.md) files (.dmg).

## Overview

ADC compressed data consist of:

* one or more [chunks](#adc_chunk)

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | big-endian |

## ADC chunk {#adc_chunk}

An ADC chunk is of variable size and consists of consists of:

<!-- rumdl-disable MD033 MD056 -->

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0.0 | 1 bit | | Literal chunk flag |
| <td colspan="4">*If literal chunk flag is set (1)*</td> |
| 0.1 | 7 bits | | Literal data size, in number of bytes, where `size = value + 1` |
| 1.0 | ... | | Literal (uncompressed) data |
| <td colspan="4">*If literal chunk flag is not set (0)*</td> |
| 0.1 | 1 bit | | Extended-size chunk flags |
| <td colspan="4">*If extended-size chunk flag is not set (0)*</td> |
| 0.2 | 4 bits | | Compressed data size, in number of bytes, where `size = value + 3` |
| 0.6 | 10 bits | | Compressed data distance, where 0 is the offset of the last previously uncompressed byte |
| <td colspan="4">*If extended-size chunk flag is set (1)*</td> |
| 0.2 | 6 bits | | Compressed data size, in number of bytes, where `size = value + 4` |
| 1.0 | 16 bits | | Compressed data distance, where 0 is the offset of the last previously uncompressed byte |

<!-- rumdl-enable MD033 MD056 -->
