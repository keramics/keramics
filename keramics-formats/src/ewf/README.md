# ewf

The ewf module provides read-only support for the
[Expert Witness Compression Format (EWF)](https://keramics.github.io/ewf.html).

Supported features:

| Category | Feature(s) |
| --- | --- |
| Format version | 1 |
| Format variants | EWF-E01 (.E01), EWF-S01 (.s01) (or SMART) |
| Compression | zlib |

Unsupported features:

| Category | Feature(s) |
| --- | --- |
| Format version | 2 (EWF2-Ex01 (.Ex01), EWF2-Lx01 (.Lx01)) |
| Format variants | EWF-L01 (.L01), EWF-X (.E01) |
| Compression | bzip2 |
| | Encryption |
