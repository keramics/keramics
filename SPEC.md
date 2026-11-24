# Keramics Implementation Specification

This document describes how to add a new format to `keramics-formats`, define binary on-disk
structures with `LayoutMap`, and integrate a new format into `keramics-vfs` and downstream crates.
It is written as a step-by-step reference for both human contributors and coding agents.

Follow [STYLE_GUIDE.md](STYLE_GUIDE.md) for Rust formatting, naming, type annotations, error
handling, docstring phrasing, and Markdown conventions.

## Workspace Architecture

Formats flow through the workspace in layers supported by dedicated utility crates:

| Crate / Path | Responsibility |
| --- | --- |
| `keramics-core` | `DataStream` trait, `DataStreamReference`, `ErrorTrace`, `Mediator`, and debug-trace macros |
| `keramics-types` | `ByteString`, `Ucs2String`, `Utf16String`, `Uuid`, and `bytes_to_*` conversion macros |
| `keramics-encodings` | `CharacterEncoding` and narrow/codepage character string decoders |
| `keramics-datetime` | `DateTime` and format-specific timestamp types (`PosixTime32`, `PosixTime64Ns`, `Filetime`, `HfsTime`, `ApfsTime`, `FatTimeDate`, etc.) |
| `keramics-checksums` | Checksum algorithms (Adler-32, CRC-32, CRC-32C / `ReversedCrc32Context`, Fletcher-64, etc.) |
| `keramics-compression` | Pure-Rust decompression contexts (DEFLATE, zlib, bzip2, LZFSE, LZVN, LZNT1, LZXPRESS) |
| `keramics-encryption` | Cipher and KDF primitives (AES-CBC/XTS/ECB, PBKDF2, key unwrap, etc.) |
| `keramics-hashes` | Cryptographic digest primitives (MD5, SHA-1, SHA-256, SHA-512, HMAC) |
| `keramics-sigscan` | Byte-pattern signature scanner (`Scanner`, `Signature`, `PatternType`) |
| `keramics-layout-map` | `#[derive(LayoutMap)]` proc macro generating debug-trace and stream-reading methods |
| `keramics-formats` | Read-only format parsers, `FormatIdentifier`, and `FormatScanner` |
| `keramics-vfs` | Unified virtual file system (`VfsType`, `VfsFileSystem`, `VfsFileEntry`, `VfsDataFork`, `VfsScanner`, `VfsCredentialStore`) |
| `keramics-tools` | CLI tools (`keramicsinfo`, `keramicsimage`, `keramicsexport`, `keramicshash`, `keramicsutil`) |
| `keramics-python` | PyO3 Python bindings exposing `keramics-vfs` types (`PyVfsType`, etc.) |
| `docs/formats` | mdBook format specifications (`docs/formats/src/<format>.md`) |

## 1. Adding a New Format (`keramics-formats`)

### 1.1 Format Specification and Test Data

Before writing parser code, document the binary specification and prepare deterministic fixtures:

* Add the binary specification in `docs/formats/src/<format>.md` and register it under its category
  in `docs/formats/src/SUMMARY.md`.
* Store minimal deterministic test images under `test_data/<format>/`.
* Where applicable, add a reproducible image-generation script under
  `scripts/generate_<format>_test_data_<os>.sh` (or `.bat` / `.ps1` on Windows; shortened format
  prefixes such as `luks` and `lvm` are also used).
* Load fixtures in unit tests via `crate::tests::get_test_data_path("<format>/<file>")` and
  `keramics_core::open_os_data_stream` (or synthetic buffers via
  `keramics_core::open_fake_data_stream`, or multi-file fixtures via `OsFileResolver` /
  `FakeFileResolver`).

### 1.2 Module Structure by Format Category

Create a new module directory at `keramics-formats/src/<format>/` modeled on an existing format of
the same category:

| Category | Reference Module(s) | Typical Public Types |
| --- | --- | --- |
| File system | `ext`, `xfs`, `ntfs`, `hfs`, `fat`, `exfat` | `<Fmt>FileSystem`, `<Fmt>FileEntry`, `<Fmt>FileEntriesIterator`, optional `<Fmt>DataFork`, `<Fmt>ExtendedAttribute`, and `<Fmt>ExtendedAttributesIterator` |
| Volume / partition system | `bsdlabel`, `gpt`, `mbr`, `apm`, `linuxlvm` | `<Fmt>VolumeSystem`, `<Fmt>Partition`, `<Fmt>PartitionsIterator` |
| Storage media image | `qcow`, `vhd`, `vhdx`, `vdi`, `vmdk`, `ewf`, `udif` | `<Fmt>Image`, `<Fmt>File` (or layer type) |
| Encrypted volume | `bde`, `luksde` | `<Fmt>EncryptedVolume`, `<Fmt>Credential` |
| Archive | `cpio` | `<Fmt>Archive`, `<Fmt>FileEntry` |
| Internal / companion data format | `decmpfs`, `plist`, `xml`, `cdsaencr` | Format-specific parser structs used by parent formats |

If a format cleanly separates its physical storage layout from its logical metadata layer, place the
physical layout types in a sibling companion module and reuse them from the logical format module.

#### Scope Exemptions for Internal, Companion, and Non-Binary Formats

Not every module requires full registration across every layer:

* Internal helper formats (`decmpfs`, `plist`, `xml`) stay private (`mod <format>;`) in `lib.rs`
  and do not need a `FormatIdentifier`, `VfsType`, or `keramicsinfo` entry.
* Companion physical layout modules and embedded encrypted wrappers (`cdsaencr`, unwrapped inside
  `udif` / `sparseimage`) do not need standalone `VfsType` variants.
* Archive formats (`cpio`) may expose `keramics-formats` and `keramicsinfo` support without a
  `VfsType` unless VFS mounting is desired.
* Text, directory-bundle, or segment-only formats (`plist`, `xml`, `sparsebundle`, `splitraw`) do
  not have fixed binary record headers and do not use `LayoutMap`.

### 1.3 Core Implementation Conventions

* **Single-stream vs. multi-file opening**:
  * Single-stream formats (file systems, single-file partition tables, encrypted volumes) implement
    `<Fmt>::new() -> Self` and
    `read_data_stream(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace>`.
  * Multi-file, segmented, or backing-chain formats (`ewf`, `linuxlvm`, `pdi`, `qcow`,
    `sparsebundle`, `splitraw`, `udif`, `vdi`, `vhd`, `vhdx`, `vmdk`, `volsnap`) implement
    `open(&mut self, file_resolver: &FileResolverReference, file_name: &PathComponent)` returning
    `Result<(), ErrorTrace>`. Resolve sibling segments and parent layers through
    `file_resolver.get_data_stream(&path_components)` so the format works identically over OS files
    (`OsFileResolver`), synthetic unit tests (`FakeFileResolver`), and VFS layers
    (`new_vfs_file_resolver`). Differential image layers link to their parent layer via
    `set_parent(&mut self, parent: &Arc<...>) -> Result<(), ErrorTrace>` (or `set_backing_file` in
    `qcow`).
* **File system entry points**:
  * Expose `get_root_directory(&self)`, `get_file_entry_by_path(&self, path: &Path)`,
    `get_file_entry_by_identifier(&self, ...)` (when applicable), and volume metadata getters.
  * Support configurable character encoding with `set_character_encoding(&mut self, encoding)`
    (returning `Result<(), ErrorTrace>` and rejecting changes after `data_stream` is opened) when
    filenames use narrow strings.
  * Formats with dual or UTF-16/UCS-2 string types define a format string enum (such as `FatString`
    or `HfsString`) or return `Ucs2String` / `Utf16String`, converting to `PathComponent` at the VFS
    boundary.
  * Case-insensitive file systems (`apfs`, `exfat`, `fat`, `hfs`, `ntfs`) use
    `PathCharacterMappings` (`Ucs2` or `Utf16`, `path_character_mappings.rs`) with
    `Path::new_with_case_folding()` and `PathComponent::new_with_case_folding()` for normalized path
    lookups.
* **Volume system entry points**:
  * Partition and volume systems whose on-disk structures depend on media sector size (`gpt`, `mbr`)
    expose `set_bytes_per_sector(&mut self, bytes_per_sector: u16) -> Result<(), ErrorTrace>` so
    callers can pass the sector size reported by an outer storage media image.
* **Traits and reusable primitives** (`keramics-formats/src/`):
  * `BlockReader` + `BlockStream<R>` (`traits.rs`, `block_stream.rs`): implement `get_size` and
    `read_data_from_blocks`, and alias `pub type <Fmt>BlockStream = BlockStream<<Fmt>BlockReader>;`
    for extent- or block-mapped data.
  * Direct `DataStream` implementations: when a format manages custom buffering, resident data,
    inline decompression, or decryption (for example, `ntfs` resident/compressed forks, encrypted
    volume streams, or multi-layer image files such as `ewf`, `qcow`, and `vmdk`), implement
    `keramics_core::DataStream` directly on the stream or file type instead of wrapping
    `BlockStream` or `RangeStream`.
  * `BlockTree<T>` (`block_tree.rs`, `block_tree_node.rs`): B-tree-like index mapping logical byte
    offsets to block/extent descriptors, used by differential/sparse images and extent file systems
    (`pdi`, `qcow`, `vdi`, `vhd`, `vhdx`, `vmdk`, `volsnap`, `xfs`).
  * `LruCache<K, V>` (`lru_cache.rs`): bounded cache for decompressed blocks, decrypted sectors,
    lookup tables, or open segment streams (`apfs`, `bde`, `ewf`, `hfs`, `luksde`, `ntfs`, `qcow`,
    `udif`, `vmdk`, etc.).
  * `RangeStream` (`range_stream.rs`): contiguous slice-backed `DataStream` for partitions and
    sub-ranges.
  * `FileResolver` / `FileResolverReference` (`file_resolver.rs`, `os_file_resolver.rs`,
    `fake_file_resolver.rs`): abstraction for opening sibling segments and backing files across OS
    paths (`OsFileResolver`), synthetic tests (`FakeFileResolver`), and VFS layers.
  * `Path` and `PathComponent` (`path.rs`, `path_component.rs`): encoding-preserving path types
    supporting `ByteString`, `String`, `Ucs2String`, and `Utf16String` components.
  * `FileEntryIterator` (`traits.rs`, `iterators.rs`): implement on `<Fmt>FileEntry` and expose
    `<Fmt>FileEntriesIterator<'a>` as an alias of `FileEntriesIterator<'a, <Fmt>FileEntry>`.
  * `ExtendedAttributeIterator` (`traits.rs`, `iterators.rs`): implement on `<Fmt>FileEntry` when
    the format supports extended attributes, and expose `<Fmt>ExtendedAttributesIterator<'a>` as an
    alias of `ExtendedAttributesIterator<'a, <Fmt>FileEntry>`.
  * `PartitionIterator` (`traits.rs`, `iterators.rs`): implement on `<Fmt>VolumeSystem` and expose
    `<Fmt>PartitionsIterator<'a>` as an alias of
    `PartitionsIterator<'a, <Fmt>VolumeSystem, <Fmt>Partition>`.
  * `IndexedHashMap` (`indexed_hash_map.rs`): insertion-ordered map with O(1) lookup by name and
    O(1) access by index for directory entries and extended attributes.
  * `calculate_alignment_padding` (`util.rs`): helper for power-of-two record alignment padding.
* **Structured text metadata parsers (`pest`)**:
  * Formats with structured text, XML, or JSON metadata (`linuxlvm/metadata.pest`,
    `luksde/metadata.pest`, `plist/xmlplist.pest`, `xml/xml.pest`) define a `.pest` PEG grammar and
    derive a parser with `#[derive(Parser)]` (`pest` / `pest_derive`) instead of hand-written
    ad-hoc string splitting.
* **Compression, checksums, and cryptography**:
  * For DEFLATE and zlib streams, use `crate::deflate_decompress!` and `crate::zlib_decompress!`
    (`keramics-formats/src/macros.rs`), which select `zlib-rs` when the optional `zlib` Cargo
    feature is enabled and fall back to `keramics-compression` otherwise.
  * Use `keramics-checksums`, `keramics-compression`, `keramics-encryption`, and `keramics-hashes`
    rather than duplicating algorithm implementations inside a format module.
* **Encryption, locking, and credentials**:
  * Encrypted formats (`apfs`, `bde`, `cdsaencr`, `luksde`, `qcow`, `sparsebundle`, `sparseimage`,
    `udif`) define a `<Fmt>Credential` enum (for example, `Passphrase`, `KeyData`,
    `RecoveryPassword`) and expose `is_locked(&self) -> bool` and
    `unlock(&mut self, credentials: &[<Fmt>Credential]) -> Result<bool, ErrorTrace>` (or
    `Result<(), ErrorTrace>`).
  * Opening an encrypted format must succeed in a locked state when no matching credentials are
    present so callers and scanners can inspect header metadata and report locked nodes.
* **Error handling, I/O, and runtime diagnostics**:
  * Return `Result<T, ErrorTrace>` for all fallible methods.
  * Use `keramics_core::data_stream_read_exact_at_position!` for positioned reads.
  * In production code, prefer explicit `match` blocks with `keramics_core::error_trace_add_frame!`
    over bare `?` so error traces preserve context at every frame.
  * Consult `keramics_core::mediator::Mediator::current()` when emitting optional runtime debug
    output (`mediator.debug_output`).
* **Module `README.md`**:
  * Create `keramics-formats/src/<format>/README.md` with a one-sentence read-only support statement
    linking to `https://keramics.github.io/<format>.html` (when a specification page exists),
    followed by `Supported features:` and (when applicable) `Unsupported features:` tables
    (`| Category | Feature(s) |`).
  * In `Unsupported features:`, list concrete unsupported format features or variants rather than
    generic catch-all rows for unsupported sector sizes (such as "Sector sizes: Others"); omit the
    `Unsupported features:` section when no specific unsupported features apply.
  * Add the format alphabetically to `keramics-formats/README.md`, linking to
    `keramics-formats/src/<format>/README.md`.

### 1.4 Registering the Format in `keramics-formats`

Update these files in alphabetical order within their sections:

1. `keramics-formats/src/lib.rs`: add `pub mod <format>;` under the appropriate category comment.
2. `keramics-formats/src/enums.rs`:
   * Add `FormatIdentifier::<Fmt>`.
   * Update `FormatIdentifier::is_storage_media_image_format()` if it is an image format.
   * Add the display string in `impl fmt::Display for FormatIdentifier` and add a test case.
3. `keramics-formats/src/scanner.rs`:
   * Import `use super::<format>::constants::*;`.
   * Add `pub fn add_<format>_signatures(&mut self)` registering header-bound
     (`PatternType::BoundToStart`) or footer-bound (`PatternType::BoundToEnd`) signatures.
   * Map every signature identifier to `FormatIdentifier::<Fmt>` in `scan_data_stream()`.
   * Register `add_<format>_signatures()` in `tests::build_all_signatures_scanner()` and add
     signature detection cases to `tests::test_scan_data_stream()`.

### 1.5 Adding `keramics-tools` Support (`keramicsinfo` and `StorageMediaImage`)

1. **`keramics-tools/src/enums.rs`**: add `<Fmt>` to `FormatType` (use `#[value(alias("..."))]` if
   the format has a well-known CLI alias).
2. **`keramics-tools/src/info/<format>.rs` and `keramics-tools/src/info/mod.rs`**: implement
   `<Fmt>Info` with `open_*` and `print_*` methods (`print_file_system` / `print_volume_system` /
   `print_image`, plus `print_file_entry_by_identifier`, `print_file_entry_by_path`, and
   `print_hierarchy` for file systems) using tab-aligned `Display` formatter structs, and export
   `<Fmt>Info` in `info/mod.rs`.
3. **`keramics-tools/src/storage_media_image.rs`** (for storage media images and encrypted volumes):
   * Wire `FormatIdentifier::<Fmt>` into `StorageMediaImage::open()` (handling multi-layer
     `image_layer` selection and credential unlocking where applicable). This enables
     `keramicsinfo --contents` and direct image opening across CLI tools.
4. **`keramics-tools/src/info_tool.rs`**:
   * Register `format_scanner.add_<format>_signatures()` in `scan_for_formats()`.
   * If the format can legitimately coexist with an outer container or volume system signature at
     offset 0 (for example, `{Bde, Fat}`, `{Volsnap, Ntfs}`, `{Udif, Hfs}`, or `{Vhd, Ext}`), add
     the disambiguation rule in `InfoTool::check_scan_results()` and a matching unit test in
     `test_check_scan_results()`.
   * If the format requires directory-extension or non-signature detection (such as
     `.sparsebundle`), handle it alongside existing path checks before falling back to
     `scan_for_formats()`.
   * Wire `FormatType::<Fmt>` and `FormatIdentifier::<Fmt>` into every `cli.command` arm (`None`,
     `Hierarchy`, `Path`, `Identifier`).
   * Add test coverage in `test_scan_for_formats_all_formats()`.
5. **`keramics-tools/tests/cli_tools.rs`**:
   * Add CLI integration test cases under the matching test function
     (`test_keramicsinfo_file_systems`, `test_keramicsinfo_volume_systems_and_archives`,
     `test_keramicsinfo_storage_media_images`, `test_keramicsinfo_auto_detect`, and
     `test_keramicsimage_commands`).

## 2. Using `LayoutMap` (`keramics-layout-map`)

Binary on-disk structures in `keramics-formats` use `#[derive(LayoutMap)]` and `#[layout_map(...)]`
above the struct doc comment. `LayoutMap` documents exact wire layouts and generates:

* `#[cfg(feature = "debug-trace")] pub fn debug_read_data(data: &[u8]) -> String`
* Optionally `read_at_position(&mut self, data_stream, position) -> Result<(), ErrorTrace>`, which
  reads a fixed-size stack buffer, emits `debug_trace_data_and_structure!`, and delegates to
  `self.read_data(&data)`.

`LayoutMap` does **not** generate `read_data`; the struct still implements `read_data` explicitly
with validation and field extraction.

> Note that `#[layout_map(bitmap(...))]` is unfinished (`TODO` in `keramics-layout-map`), and
> `member(...)` / `method(...)` are legacy forms. Always use `structure(...)` with `field(...)` /
> `group(...)` and `methods(...)`.

### 2.1 Attribute Grammar and Supported Options

```rust
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 4]", format = "hex"),
        field(name = "block_size", data_type = "u32"),
        field(name = "last_written_time", data_type = "PosixTime32"),
        field(name = "volume_label", data_type = "ByteString<32>"),
        group(
            size_condition = ">= 64",
            field(name = "extended_flags", data_type = "u32", format = "hex"),
        ),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Example structure.
pub struct ExampleHeader {
    // ...
}
```

#### Structure, Field, and Group Attributes

| Attribute | Scope | Accepted Values | Notes |
| --- | --- | --- | --- |
| `byte_order` | `structure`, `field` | "little" ("le", "LittleEndian"), "big" ("be", "BigEndian") | Required on `structure` or on each multi-byte `field`; a `field` may override the structure default |
| `name` | `field` | Non-empty string | Field name shown in debug-trace output |
| `data_type` | `field` | See supported data types table below | Determines byte offset advancement and formatting |
| `format` | `field` | "hex" ("Hexadecimal"), "char" ("Character") | Optional display formatting override |
| `modifier` | `field` | Expression suffix string (for example, "+ 1") | Optional display adjustment for bitfields or integers |
| `size_condition` | `group` | Comparison suffix against `data.len()` (for example, ">= 64", "> 32") | Conditionally traces trailing fields when the buffer is large enough |
| `methods(...)` | top-level | "debug_read_data", "read_at_position" | Only use "read_at_position" when `self.read_data(&data)` takes `(&mut self, &[u8])` with no extra parameters and fixed byte length |

#### Supported `data_type` Values

| Category | Syntax | Size |
| --- | --- | --- |
| Unsigned integers | "u8" / "u16" / "u32" / "u64" (or "uint8".."uint64", "UnsignedInteger*Bit") | 1, 2, 4, 8 bytes |
| Signed integers | "i8" / "i16" / "i32" / "i64" (or "int8".."int64", "SignedInteger*Bit") | 1, 2, 4, 8 bytes |
| Fixed-length arrays | "[u8; N]", "[u16; N]", "[u32; N]", "[u64; N]" (any supported element type) | `N * sizeof(T)` |
| Bitfields | "BitField8<bits>", "BitField16<bits>", "BitField32<bits>", "BitField64<bits>", "BitField128<bits>" | Consecutive bitfields must fill the full container width before the next non-bitfield `field` |
| Strings | "ByteString<N>", "Ucs2String<N>", "Utf16String<N>" | `N` bytes (ByteString) or `N * 2` bytes (UCS-2 / UTF-16) |
| Timestamps | "PosixTime32", "Filetime", "HfsTime", "ApfsTime", "FatDate", "FatTimeDate", "FatTimeDate10Ms" | Fixed per timestamp type |
| UUID | "Uuid" or "uuid" | 16 bytes (can specify field-level `byte_order` as "big" or "little") |
| Nested struct | "Struct<TypeName; Size>" | `Size` bytes; invokes `TypeName::debug_read_data(...)` |

### 2.2 Structural Patterns

#### Fixed-Size Single-Endian Structure

For a single-version, single-endian structure whose `read_data(&mut self, data: &[u8])` needs no
extra arguments (for example, `ExtSuperblock`), derive `LayoutMap` directly on the struct with
`methods("debug_read_data", "read_at_position")`.

#### Nested Structs and Bitfields

Use `Struct<TypeName; Size>` to embed another `LayoutMap` type (`mbr/partition_entry.rs`,
`xfs/superblock.rs`) and `BitField*<bits>` for packed integers (`xfs/packed_extent.rs`,
`apfs/object_header.rs`). Consecutive bitfields are ordered from least-significant bits to
most-significant bits and must sum exactly to the container width (8, 16, 32, 64, or 128 bits):

```rust
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "start_address_chs", data_type = "Struct<MbrChsAddress; 3>"),
    ),
    methods("debug_read_data")
)]
pub struct ExamplePartitionEntry {}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "number_of_blocks", data_type = "BitField128<21>"),
        field(name = "physical_block_number", data_type = "BitField128<52>"),
        field(name = "logical_block_number", data_type = "BitField128<54>"),
        field(name = "uninitialized_flag", data_type = "BitField128<1>"),
    ),
    methods("debug_read_data")
)]
pub struct ExamplePackedExtent {}
```

#### Versioned Structures (Marker Struct Pattern)

When an on-disk structure has multiple incompatible layouts across format versions (for example,
`Ext2Inode` / `Ext3Inode` / `Ext4Inode` or `Ext2GroupDescriptor` / `Ext4GroupDescriptor`):

1. Define a unified domain struct (`ExtInode`) without `#[derive(LayoutMap)]`.
2. Define one empty marker struct per on-disk layout in its own file (`inode_ext2.rs`,
   `inode_ext3.rs`, `inode_ext4.rs`) with `#[derive(LayoutMap)]` and `methods("debug_read_data")`.
3. Implement `pub fn read_data(target: &mut UnifiedStruct, data: &[u8], ...)` returning
   `Result<(), ErrorTrace>` on each marker struct.
4. Provide a `#[cfg(feature = "debug-trace")] pub fn debug_read_data(...) -> String` dispatcher on
   the unified struct and call it from the reader via
   `keramics_core::debug_trace_data_and_structure!` or `keramics_core::debug_trace_structure!`.

#### Dual-Endian Structures

Because `LayoutMap` resolves `byte_order` at macro expansion time, dual-endian formats (such as
`cpio`) use one of two approaches:

* **Separate LE and BE marker structs with `read_data`** (when parsing logic differs or is split by
  endianness, as in `cpio/file_record_bin_le.rs` and `cpio/file_record_bin_be.rs`).
* **Primary LE layout plus `#[cfg(feature = "debug-trace")]` BE helper struct** (when `read_data`
  accepts a runtime byte-order parameter):

```rust
#[cfg(feature = "debug-trace")]
#[derive(LayoutMap)]
#[layout_map(
    structure(byte_order = "big", /* fields */),
    methods("debug_read_data")
)]
/// Big-endian debug layout.
struct ExampleHeaderBigEndian {}

#[derive(LayoutMap)]
#[layout_map(
    structure(byte_order = "little", /* fields */),
    methods("debug_read_data")
)]
/// Example header.
pub struct ExampleHeader {
    // ...
}
```

> Note that gating debug-only alternate layout structs (`*BigEndian` or legacy variant layouts) with
> `#[cfg(feature = "debug-trace")]` prevents dead-code warnings when building `keramics-formats`
> with default features (without `debug-trace`).

### 2.3 Wiring Debug-Trace Macros

Use the `keramics_core` debug-trace macros at stream and buffer read sites:

* `keramics_core::debug_trace_data_and_structure!(...)` when reading a structure directly from a
  `DataStreamReference` (passing "TypeName", `offset`, `&data`, `size`, and
  `Type::debug_read_data(...)`).
* `keramics_core::debug_trace_structure!(Type::debug_read_data(...))` when parsing an entry from an
  already-traced buffer (for example, directory entries or extended attribute records inside a
  block).
* `keramics_core::debug_trace_data!("TypeName", offset, &data, size)` for raw buffers without a
  fixed structure header.

Add `#[cfg(feature = "debug-trace")]` unit test assertions that invoke every `debug_read_data*`
helper to verify field offsets do not panic on out-of-bounds slices.

## 3. Adding a New VFS Implementation (`keramics-vfs`)

### 3.1 Choosing the Integration Pattern

Select the `keramics-vfs` pattern based on the format category:

| Category | Pattern | Reference |
| --- | --- | --- |
| File system | Wrap `<Fmt>FileSystem`, `<Fmt>FileEntry`, and optional `<Fmt>ExtendedAttribute` / `<Fmt>DataFork` directly in `VfsFileSystem`, `VfsFileEntry`, `VfsExtendedAttribute`, and `VfsDataFork` (no submodule needed) | `ext`, `xfs`, `exfat`, `fat`, `hfs`, `ntfs` |
| Volume / partition system | Create `keramics-vfs/src/<format>/` using `VfsPartitionFileSystem<P, S>` and `VfsPartitionFileEntry<P, S>` by implementing `VfsPartition` (`NAME_PREFIX`, `get_data_stream`, `get_identifier`, `get_partition_number`, `get_partition_size`) and `VfsPartitionSystem` (`PATH_PREFIX`, `new`, `open_from_vfs`) from `keramics-vfs/src/traits.rs` | `bsdlabel`, `gpt`, `mbr`, `apm`, `sgilabel` |
| Storage media image | Create `keramics-vfs/src/<format>/` using `VfsImageFileSystem<I, L>` and `VfsImageFileEntry<I, L>` by implementing `VfsImage` (`PATH_PREFIX`, `Layer`, `new`, `get_bytes_per_sector`, `get_number_of_layers`, `get_layer_by_index`, `open_from_vfs`) and `VfsImageLayer` (`NAME_PREFIX`, `get_data_stream`, `get_identifier`, `get_media_size`) from `keramics-vfs/src/traits.rs` | `qcow`, `vhd`, `vhdx`, `vdi`, `vmdk`, `ewf`, `pdi`, `udif` |
| Encrypted / hybrid container | Create a dedicated `keramics-vfs/src/<format>/` submodule integrating with `VfsCredentialStore` (`VfsCredentialStore::current()`) to map `VfsCredential` entries and unlock encrypted volumes or layers | `bde`, `luksde`, `apfs` |

### 3.2 `keramics-vfs` File-by-File Checklist

1. **`keramics-vfs/src/enums.rs`**:
   * Add `VfsType::<Fmt>` alphabetically.
   * Add its display name in `impl fmt::Display for VfsType` and add an assertion in
     `tests::test_vfs_type_fmt()`.
2. **`keramics-vfs/src/lib.rs`** (only if adding a new wrapper submodule):
   * Declare `mod <format>;` alphabetically.
3. **`keramics-vfs/src/data_fork.rs`** (only if the format supports multiple forks or named data
   streams, such as `hfs` or `ntfs`):
   * Add `VfsDataFork::<Fmt>(<Fmt>DataFork)`.
   * Handle `get_data_stream()` and `get_name()`.
   * Add unit tests.
4. **`keramics-vfs/src/extended_attribute.rs`** (only if the format supports extended attributes):
   * Add `VfsExtendedAttribute::<Fmt>(<Fmt>ExtendedAttribute)`.
   * Handle `get_data_stream()` and `get_name()` (matching custom string types if applicable).
   * Add unit tests if test fixtures contain extended attributes.
5. **`keramics-vfs/src/file_system.rs`**:
   * Add `VfsFileSystem::<Fmt>(<Fmt>FileSystem)`.
   * Update `new()`, `file_entry_exists()`, `get_file_entry_by_path()`, `get_root_file_entry()`,
     `get_type()`, and `open()`.
   * For direct file systems, add `fn open_<format>_file_system(...)` that retrieves the parent data
     stream via `file_system.get_data_stream_by_path_and_name(path, None)` and calls
     `read_data_stream(&data_stream)`.
   * Add unit tests (`file_entry_exists`, `get_file_entry_by_path` for root, file, and non-existing
     path) and add `VfsType::<Fmt>` to `test_get_type()`, `test_get_root_file_entry()`, and
     `test_open_errors()`.
6. **`keramics-vfs/src/file_entry.rs`**:
   * Add `VfsFileEntry::<Fmt>(<Fmt>FileEntry)`.
   * Update every match arm: `get_access_time`, `get_change_time`, `get_creation_time`,
     `get_device_identifier`, `get_file_mode`, `get_file_type`, `get_modification_time`,
     `get_group_identifier`, `get_inode_number`, `get_name` (converting `ByteString`, `Ucs2String`,
     `Utf16String`, or format string enums to `PathComponent`), `get_number_of_links`,
     `get_owner_identifier`, `get_size`, `get_symbolic_link_target`, `get_type`,
     `get_number_of_data_forks`, `get_data_fork_by_index`, `get_data_stream`,
     `get_data_stream_by_name`, `get_extended_attribute_by_name`, `is_root_directory`,
     `get_number_of_extended_attributes`, `get_extended_attribute_by_index`,
     `get_sub_file_entry_by_index`, and `get_number_of_sub_file_entries`.
   * Add unit tests covering metadata, data forks/streams, symbolic links, sub-entry iteration,
     extended attributes, `test_get_type_and_is_root_directory()`, and
     `test_get_file_type_special_files()`.
7. **`keramics-vfs/src/scanner/scan_node.rs`**:
   * Add `VfsType::<Fmt>` to `VfsScanNode::is_file_system()` (which intentionally has no wildcard
     arm) and `tests::test_is_file_system()`.
8. **`keramics-vfs/src/scanner/scanner.rs`**:
   * Register `add_<format>_signatures()` in `VfsScanner::build()` on the appropriate phased
     scanner:
     * `storage_media_image_scanner`: storage media image formats.
     * `phase1_volume_system_scanner`: non-overlapping volume systems (`apfs`, `apm`, `bde`,
       `bsdlabel`, `gpt`, `linuxlvm`, `luksde`, `sgilabel`, `volsnap`).
     * `phase2_volume_system_scanner`: exclusion signatures that suppress false MBR matches
       (`exfat`, `fat`, `ntfs`).
     * `phase3_volume_system_scanner`: overlapping volume systems (`mbr`, with fallback structural
       validation for formats without unique signatures such as `fat`).
     * `sub_volume_system_scanner`: volume systems that can appear inside partitions or images.
     * `file_system_scanner`: file system formats (`exfat`, `ext`, `fat`, `hfs`, `ntfs`, `xfs`).
   * For non-signature formats, add path/directory heuristics (such as directory `.sparsebundle`
     extension checks in `VfsScanner::scan()` or segmented `.000` naming checks for `splitraw`).
   * Map `FormatIdentifier::<Fmt> => Some(VfsType::<Fmt>)` in `VfsScanner::get_vfs_type()` and add
     it to `tests::test_get_vfs_type()`. Note that `FormatIdentifier` and `VfsType` variant names
     are not always 1:1 (for example, `FormatIdentifier::Luks` maps to `VfsType::Luksde`, and
     `FormatIdentifier::Apfs` maps to `VfsType::ApfsContainer`, which in turn exposes inner
     `VfsType::Apfs` volumes).
   * Update `scan_for_format()` and `scan_sub_node()` for `VfsType::<Fmt>` (including marking
     `scan_node.is_locked = true` or calling `add_locked_sub_node()` when encrypted volumes remain
     locked).
   * Guard against self-recursion when a partition can start at offset 0 of its parent volume system
     (for example, `VfsType::BsdDiskLabel` ignores a self-matching `FormatIdentifier::BsdDiskLabel`
     so the partition falls through to `scan_for_file_system_format()`).
   * Add scanner tests (`test_scan_for_*_format_with_<format>` and end-to-end `scan()` coverage).

### 3.3 Required Downstream Updates

Adding a new `VfsType` or `VfsFileEntry` variant requires updating downstream consumers:

1. **`keramics-tools/src/display_path.rs`**:
   * In `get_path()` under `OutputFormat::DfImageTools`, add file systems and volume containers
     (`Apfs`, `ApfsContainer`, `ExFat`, `Ext`, `Fat`, `Hfs`, `LinuxLvm`, `Ntfs`, `Volsnap`, `Xfs`)
     to concatenate with the parent display path, and add partition tables (`Apm`, `BsdDiskLabel`,
     `Gpt`, `Mbr`, `SgiDiskLabel`) to emit the partition path directly. Storage media images and
     encrypted volumes intentionally fall through to `_ => None`.
   * In `get_path_component()`, add `VfsFileEntry::<Fmt>` under `DisplayPathType::Identifier` for
     formats exposing UUID/GUID layer or volume identifiers (`ApfsContainer`, `Gpt`, `LinuxLvm`,
     `Pdi`, `Vhd`, `Vhdx`, `Volsnap`), and under `DisplayPathType::Index` for partition tables
     (`BsdDiskLabel`, `Gpt`, `Mbr`, `SgiDiskLabel`).
2. **`keramics-tools/src/image_tool.rs`**: for file systems, handle `VfsFileEntry::<Fmt>` when
   formatting the inode/identifier column and POSIX file mode string.
3. **`keramics-python/src/vfs.rs`**:
   * Add `#[pyo3(name = "<UPPER_SNAKE>")] <Fmt>` to `PyVfsType`.
   * Add `PyVfsType::<Fmt> => VfsType::<Fmt>` in `impl From<&PyVfsType> for VfsType`.
   * Add an assertion in `tests::test_vfs_type_from_py_vfs_type()`.
   * Optionally add end-to-end test cases in `keramics-python/tests/test_vfs.py`.

## 4. Verification Checklist

Run the following checks before considering a new format or VFS implementation complete:

```bash
# Formatting check
cargo fmt --all && cargo fmt --check

# Verify zero warnings with default features (without debug-trace or zlib) and all features
cargo check -p keramics-formats
cargo check -p keramics-formats --all-features

# Full workspace test suite (includes LayoutMap debug-trace, zlib, and keramics-python Rust tests)
cargo test --workspace --all-features

# CLI smoke tests
cargo run -p keramics-tools --bin keramicsinfo -- test_data/<format>/<sample>
cargo run -p keramics-tools --bin keramicsimage -- test_data/<format>/<sample>

# Documentation and spelling checks (when installed)
rumdl check
typos
mdbook build docs/formats
```
