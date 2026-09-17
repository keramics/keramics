# Contributing to Keramics

Thanks for your interest in contributing to the Keramics project.

**Note that this project is in an early phase (experimental, pre-release), so the codebase is
evolving.**

## Contribution Policy

Please note that the GitHub Issue Tracker is currently reserved for contributors.

* If you want to discuss a feature or report a bug, reach out via
  [the Open Source DFIR Slack](https://github.com/open-source-dfir/slack).

## How to Help Out

If you want to become a contributor:

* Reach out via [the Open Source DFIR Slack](https://github.com/open-source-dfir/slack).

## Before you begin

By submitting an idea (Feature Request) or change (Pull Request) to this project, you agree to the
following **Individual Contributor License Agreement (CLA)**. This ensures that all copyright and
intellectual property (IP) contributed legally becomes a permanent part of the project.

This projects therefore requires you to add your legal name and a valid email address in commits.
If you do not wish to disclose such information but still want to contribute, reach out to the
maintainers.

### Individual Contributor License Agreement (CLA)

By submitting a contribution to Keramics ("the Project"), you agree to the following terms and
conditions:

1. **Definitions:**
   * "Contribution" means any source code, documentation, or other material submitted by you for
     inclusion in, or documentation of, the Project.
   * "Submit" means any form of electronic, verbal, or written communication sent to the Project
     managers, including but not limited to communication on GitHub pull requests, issue trackers,
     or email lists.
2. **Grant of Copyright License:** You hereby grant to the Project and to recipients of software
   distributed by the Project a perpetual, worldwide, non-exclusive, no-charge, royalty-free,
   irrevocable copyright license to reproduce, prepare derivative works of, publicly display,
   publicly perform, sublicense, and distribute your Contributions and such derivative works.
3. **Grant of Patent License:** You hereby grant to the Project and to recipients of software
   distributed by the Project a perpetual, worldwide, non-exclusive, no-charge, royalty-free,
   irrevocable patent license to make, have made, use, offer to sell, sell, import, and otherwise
   transfer the Project where such license applies only to those patent claims licensable by you
   that are necessarily infringed by your Contribution(s) alone or by combination of your
   Contribution(s) with the Project.
4. **Ownership of Intellectual Property:** You retain ownership of the copyright in your
   Contribution. However, you acknowledge and agree that upon submission, the IP and copyright
   rights granted in Sections 2 and 3 permanently become a part of the Project and will be licensed
   to the public under the Project's chosen open-source license.
5. **Representations:** You represent that you are legally entitled to grant these licenses. If
   your employer has rights to intellectual property you create, you represent that you have
   received permission to make Contributions on behalf of that employer or that your employer has
   waived such rights for your Contributions.
6. **Disclaimer of Warranty:** Except for the representations explicitly stated in Section 5, your
   Contributions are provided on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied, including, without limitation, any warranties or conditions of TITLE,
   NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.

## Getting Started

### Prerequisites

You need the standard Rust toolchain.

### Set up

Fork the repository on GitHub and clone your fork locally:

```bash
git clone https://github.com/username/keramics.git
cd keramics
```

Create a feature branch for your changes:

```bash
git checkout -b feature
```

## Development Workflow

This project uses standard `cargo` workflows. Please ensure your changes pass all local checks
before submitting a Pull Request (PR).

**Format Rust code.**

```bash
cargo fmt --all
```

**Build the project.**

```bash
cargo build --all-features
```

**Run the tests locally.**

```bash
cargo test --all-features
```

### Python bindings

**Format Python code.**

```bash
black .
```

**Run the tests locally.**

```bash
cd keramics-python/ && tox -epy314
```

### Bash scripts

**Check the formatting of bash scripts.**

```bash
find scripts/ -name "*.sh" -exec shellcheck {} +
```

### Documentation (Markdown)

**Check the formatting of Markdown files.**

```bash
rumdl check
```

## Adding support for a data format

**Start with test data.**

One of the focuses of this project is reproducibility and hence test data that can be generated
deterministically is preferred. Scripts to generated test data can be found in the "scripts"
directory.

If the test data cannot be created by means of script, make sure:

* you add a Markdown document that describes how the test data was generated;
* the test data has a project compatible license.

If the test data was not authored by you (as the contributor), make sure you have permission to use
it and mention its original source in "ACKNOWLEDGEMENTS.md". **Do not include test data that cannot
be redistributed.**

**Document the data format.**

Data formats change over time, hence documenting their structures is important. Data format
documentation can be found in the "docs/formats" directory and is hosted on:
[keramics.github.io](https://keramics.github.io)

This project uses [mdBook](https://rust-lang.github.io/mdBook) and therefore requires
[CommonMark](https://commonmark.org/) compliant Markdown.

**Code style.**

More details will be added at a later date but for now write your code closest to English natural
language, for an international audience, as possible.

## Submitting a Pull Request (PR)

Keep the size of a Pull Request (PR) reasonable. If you are planning substantially large changes
discuss these with the maintainers first.

1. Make sure your commit contains a legal name and valid email address.
1. Push your branch to GitHub, e.g. with `git push --set-upstream origin feature`.
1. Make sure the tests pass on GitHub.
1. Open a PR against the `main` branch. Only create a PR when your changes are ready. In your PR
   description, clearly explain what your code changes.

## Questions & Discussions

For all other questions or discussions, please reach out via
[Open Source DFIR Slack](https://github.com/open-source-dfir/slack).
