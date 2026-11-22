# Keramics Style Guide

This guide documents the code style, docstring, and documentation conventions used throughout the
Keramics codebase.

## General Principles

* Write code, identifiers, comments, and documentation as close to natural English as possible for
  an international audience.
* Use US English spelling (`locale = "en-us"`), enforced across filenames, source code, and
  documentation by `typos` (configured in `.typos.toml`).
  * When a legitimate term or binary identifier triggers a false positive, add it to `.typos.toml`
    or suppress it locally with `// typos:ignore`, `// typos:disable` / `// typos:enable`, or
    `<!-- typos:disable -->` / `<!-- typos:enable -->`.
* Keep line endings as LF (configured in `.gitattributes`, with `test_data/**` marked binary).
* Include the Apache 2.0 copyright and license header at the top of every source file and script.

### License Headers

Rust files use a C-style block comment header:

```rust
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
```

Python and Bash files use `#` comment headers beneath the shebang line (`#!/usr/bin/env python3`
or `#!/usr/bin/env bash`).

## Rust Code Style

### Formatting and Edition

* All crates use Rust edition 2024 (inherited from the workspace `Cargo.toml`).
* Format all Rust code with standard `rustfmt` (`cargo fmt --all`). CI verifies formatting with
  `cargo fmt --check`.

### Naming Conventions

* Follow standard Rust casing (`snake_case` for functions, methods, modules, and variables;
  `UpperCamelCase` for types, traits, and enum variants; `SCREAMING_SNAKE_CASE` for constants).
* Avoid cryptic or unnecessary abbreviations in variable, parameter, field, and function names:
  * Use `identifier` instead of `id`.
  * Use `initialization_vector` instead of `iv`.
  * Use `symbolic_link` instead of `symlink`.
  * Use `block_value0`, `cipher_value0`, `input_value0` instead of `b0`, `c0`, `input0`.
  * Use `number_of_<items>` for counts (for example, `number_of_entries`, `number_of_sectors`).
  * Use `standard_output` / `standard_error` instead of `stdout` / `stderr` for local variables
    where appropriate.
* Prefix format-specific types and constants with the format name (for example,
  `ApmPartitionMapEntry`, `BdeEncryptionContext`, `ExtFileEntry`, `VfsFileEntry`,
  `APM_PARTITION_MAP_SIGNATURE`).
* Common idiomatic local names used consistently across the codebase include `path_buf: PathBuf`,
  `data_stream: DataStreamReference`, `test_data`, and `test_struct`.
* Prefer byte string literals (`b"..."`, `b"...".to_vec()`, `b"...".as_slice()`) over
  `"...".as_bytes()` or ASCII byte vectors like `vec![0x74, 0x65, 0x73, 0x74]` when representing
  ASCII `ByteString` elements or byte patterns. Reserve `vec![0x80]` / `vec![0xff]` for empty or
  non-ASCII raw byte sequences.

### Type Annotations and Bindings

* Provide explicit type annotations on `let` bindings whenever the type is concrete and readable:

```rust
let data_size: usize = data.len();
let mut offset: u64 = 0;
let path: Path = Path::from("/testdir1/testfile1");
```

* Discard unused return values with `_ = expression;` rather than `let _ = expression;`.

### Imports and Module Organization

* Place all `use` statements at the top of the file or at the top of `mod tests`; do not place
  `use` statements inline inside functions.
* Group imports with blank lines between groups, sorted alphabetically within each group:
  1. `std::` imports.
  2. External and workspace crate imports (`keramics_*`, third-party crates).
  3. `crate::` imports.
  4. `super::` imports.
* Inside `#[cfg(test)] mod tests`, place `use super::*;` first, followed by the same grouping
  order (`std::`, external / `keramics_*`, `crate::`).
* Keep module declarations (`mod ...;`) and re-exports (`pub use ...;`) alphabetically sorted
  within their logical sections (for example, format categories in `keramics-formats/src/lib.rs`).

### Structs, Methods, and Layout Maps

* Define helper structs and functions at module scope (or at the top of `mod tests` for test
  helpers), not inline inside function bodies.
* Within `impl` blocks, place constructors first (`new`, `new_*`), followed by all remaining
  methods in alphabetical order.
* For binary format structures, use `#[derive(LayoutMap)]` and `#[layout_map(...)]` above the
  struct doc comment to generate debug-trace and stream-reading methods (`debug_read_data`,
  `read_at_position`).

### Error Handling and Tracing

* Return `Result<T, ErrorTrace>` for fallible operations using `keramics_core::ErrorTrace`.
* Construct errors with `keramics_core::error_trace_new!` or
  `keramics_core::error_trace_new_with_error!`.
* In production code, prefer explicit `match` blocks with `keramics_core::error_trace_add_frame!`
  over bare `?` so that error traces preserve descriptive context at each call frame:

```rust
match self.encryption_context.decrypt_sector(sector_physical_offset, &mut sector_data) {
    Ok(_) => {}
    Err(mut error) => {
        keramics_core::error_trace_add_frame!(
            error,
            format!("Unable to decrypt sector: {}", sector_number)
        );
        return Err(error);
    }
}
```

* Using `?` is acceptable in unit tests (`fn test_...() -> Result<(), ErrorTrace>`).
* Use `keramics_core::data_stream_read_exact_at_position!` for positioned reads and
  `keramics_core::debug_trace_data!` for debug tracing under the `debug-trace` feature.

### Testing Conventions

* Place unit tests in `#[cfg(test)] mod tests` at the bottom of the corresponding source file.
* Place integration and CLI tests in `<crate>/tests/`.
* Name test functions after the method and scenario under test:
  * `test_<method_name>`
  * `test_<method_name>_with_<condition>` (for example,
    `test_read_data_with_unsupported_data_size`)
* When asserting that a call returns an error, split the function call and the `.is_err()` check
  across separate lines by storing the return value in a local variable named `result`:

```rust
let result: Result<(), ErrorTrace> = test_struct.read_data(&test_data);
assert!(result.is_err());
```

* Gate test-only helper methods on production types with `#[cfg(test)]` and an inline comment
  (`// Used for testing.`) instead of `#[allow(dead_code)]`.
* Store deterministic test fixtures under `test_data/<format>/` and resolve paths via crate-level
  `get_test_data_path(...)` helpers.

## Rust Docstrings and Comments

### Module Docstrings (`//!`)

Modules that document a public subsystem or algorithm place a `//!` block immediately after the
license header:

```rust
//! Adler-32 checksum.
//!
//! Provides support for calculating Adler-32 checksums.
```

* Line 1: concise noun-phrase title ending with a period.
* Line 2: blank `//!` line.
* Line 3+: one-sentence summary starting with `Provides ...` and ending with a period.

### Item Docstrings (`///`)

Document all structs, struct fields, enums, traits, constants, type aliases, and functions or
methods with `///` comments written as concise, capitalized sentences ending with a period.

Use consistent phrasing across the codebase:

| Item kind | Convention | Example |
| --- | --- | --- |
| Struct / enum | Noun phrase describing the type. | `/// BitLocker Drive Encryption (BDE) block reader.` |
| Struct field | Short capitalized noun phrase. | `/// Bytes per sector.` |
| Constructor | `Creates a new <item>.` | `/// Creates a new encryption context.` |
| Getter | `Retrieves the <property>.` | `/// Retrieves the size of the data.` |
| Predicate | `Determines if <condition>.` | `/// Determines if the container is locked.` |
| Reader / parser | `Reads the <item> from ...` | `/// Reads the partition map entry from a buffer.` |
| `fmt::Display` | `Formats the <item> for display.` | `/// Formats the UUID for display.` |
| `From` impl | ``Converts a [`Source`] into a [`Target`].`` | ``/// Converts an [`&PyVfsType`] into a [`VfsType`].`` |

### Inline Comments (`//`)

* Write inline comments as complete sentences beginning with a capital letter and ending with a
  period (lowercase format names such as `ext` are acceptable when used as proper names).
* Use `// Note that ...` for behavioral caveats and non-obvious invariants.
* Use `// Test with ...` in test functions to separate distinct test cases.
* Keep `// TODO: ...` comments short and actionable.

## Python Code Style (`keramics-python`)

* Format Python code with `black` (`black keramics-python`; verified in CI with
  `black --check keramics-python`).
* Start test files with `#!/usr/bin/env python3` followed by the Apache 2.0 license header.
* Group imports in PEP 8 order: standard library, third-party (`pytest`), then `pykeramics`.
* Write tests using `pytest`, organizing test methods (`test_<method_name>`) into `Test<Type>`
  classes (with shared `get_test_data_path` helper on `TestClass`).
* Guard platform-specific checks with `os.name == "posix"` or `os.name == "nt"` so tests pass
  across Linux, macOS, and Windows.

## Shell Script Style (`scripts/`)

* Begin scripts with `#!/usr/bin/env bash`, followed by a one-line description comment and the
  Apache 2.0 license header.
* Ensure all `.sh` files pass `shellcheck` with the repository `.shellcheckrc` (`shell=bash`,
  `severity=warning`, `SC2181` and `SC2317` disabled).
* Source the platform helper library (`shared_linux.sh`, `shared_macos.sh`, or `shared_bsd.sh`)
  and validate required external tools with `assert_availability_binary`.
* Enable `set -e` before creating or mounting test images, and exit with `exit ${EXIT_SUCCESS}`.
* Use `UPPER_CASE` for script variables and quote expansions as `"${VARIABLE}"`.
* Document shell functions with a description comment and an `# Arguments:` block above the
  function definition.

## Documentation Conventions (Markdown)

### Markdown Formatting (`rumdl`)

All Markdown files matched by `.rumdl.toml` (`*.md`, `docs/*/src/*.md`, `keramics-*/README.md`,
and `keramics-formats/src/*/README.md`) must be CommonMark compliant and pass `rumdl check`:

* **Unordered lists (`MD004`):** Always use asterisks (`*`), never `-` or `+`.
* **Nested list indentation (`MD007`):** Indent nested list items by 2 spaces.
* **Line length (`MD013`):** Wrap prose at 100 characters (`reflow = true`); tables are exempt
  (`tables = false`).
* **Tables (`MD055`):** Use leading and trailing pipes (`| Column | Description |`) and `---`
  separators (`| --- | --- |`).

### Crate `README.md` Files

Each workspace crate (`keramics-*/README.md`) follows a minimal, uniform template:

* `# <crate-name>` heading.
* One-sentence description of the crate.
* `[docs.rs](https://docs.rs/<crate_name>)` link.
* `## License` section linking to
  [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).

### Format Module `README.md` Files

Each format module under `keramics-formats/src/<format>/README.md` documents implementation scope:

* `# <format>` heading.
* One-sentence read-only support statement linking to
  `https://keramics.github.io/<format>.html`.
* `Supported features:` and `Unsupported features:` tables with `| Category | Feature(s) |`
  columns.

### Data Format Specifications (`docs/formats`)

Format specifications live in `docs/formats/src/<format>.md`, are listed in
`docs/formats/src/SUMMARY.md` by format category, and are built with `mdBook` using the
`mdbook-tables` preprocessor:

* Start with `# <Full Format Name> (<ABBR>) format` and a short introductory paragraph linking to
  related formats.
* Include `## Overview`, a `### Characteristics` table (`Byte order`, `Date and time values`,
  `Character strings`), and an optional `### Terminology` table.
* Document binary structures with `| Offset | Size | Value | Description |` tables, using escaped
  byte signatures (for example, `"\x50\x4d" or "PM"`) and hex values (`0x00`).
* Document flag and enumeration tables with `| Value | Identifier | Description |` columns.
* Add explicit heading anchors (`{#section_name}`) when linking between sections within a
  specification.
* Use blockquotes starting with `> Note that ...` for format caveats and edge cases.
* When spanning columns in Markdown tables, use `<td colspan="N">...</td>` cells (preprocessed by
  `docs/mdbook-tables`).

## Automated Verification Checklist

Run these checks locally before submitting changes (matching `.github/workflows/lint.yml` and
`.github/workflows/build.yml`):

```bash
cargo fmt --check
cargo test --all-features
black --check keramics-python
find scripts -name "*.sh" -exec shellcheck {} +
rumdl check
typos
mdbook build docs/formats
```
