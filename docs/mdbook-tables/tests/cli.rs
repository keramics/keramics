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

use std::io::Write;
use std::process::{Child, Command, Output, Stdio};

#[test]
fn test_cli_supports() {
    let binary_path: &str = env!("CARGO_BIN_EXE_mdbook-tables");

    let output: Output = Command::new(binary_path)
        .args(["supports", "html"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn test_cli_unsupported_argument() {
    let binary_path: &str = env!("CARGO_BIN_EXE_mdbook-tables");

    let output: Output = Command::new(binary_path)
        .arg("unsupported")
        .output()
        .expect("Failed to execute binary");

    assert!(!output.status.success());

    let standard_error: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(standard_error.contains("Unsupported argument: unsupported"));
}

#[test]
fn test_cli_preprocessing() {
    let binary_path: &str = env!("CARGO_BIN_EXE_mdbook-tables");

    let input: &str = concat!(
        "[\n",
        "    {\n",
        "        \"root\": \".\",\n",
        "        \"config\": {},\n",
        "        \"renderer\": \"html\",\n",
        "        \"mdbook_version\": \"0.5.2\"\n",
        "    },\n",
        "    {\n",
        "        \"items\": [\n",
        "            {\n",
        "                \"Chapter\": {\n",
        "                    \"name\": \"Characteristics\",\n",
        "                    \"content\": \"",
        "| Characteristics | Description |\\n",
        "| --- | --- |\\n",
        "| Byte order | big-endian |\\n",
        "\",\n",
        "                    \"path\": \"intro.md\",\n",
        "                    \"sub_items\": [],\n",
        "                    \"parent_names\": []\n",
        "                }\n",
        "            }\n",
        "        ]\n",
        "    }\n",
        "]\n",
    );
    let mut child: Child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn binary");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .expect("Failed to write standard input");

    let output: Output = child.wait_with_output().expect("Failed to wait for binary");

    assert!(output.status.success());

    let standard_output: String = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(standard_output.contains("<div class=\\\"table-wrapper\\\">"));
    assert!(standard_output.contains("<tr><td>Byte order</td><td>big-endian</td></tr>"));
}

#[test]
fn test_cli_preprocessing_with_invalid_input() {
    let binary_path: &str = env!("CARGO_BIN_EXE_mdbook-tables");

    let mut child: Child = Command::new(binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn binary");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"invalid json")
        .expect("Failed to write standard input");

    let output: Output = child.wait_with_output().expect("Failed to wait for binary");

    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
}
