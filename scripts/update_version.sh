#!/usr/bin/env bash
#
# Script to update version.
#
# Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License. You may
# obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations
# under the License.

EXIT_SUCCESS=0
EXIT_FAILURE=1

if test $# -ne 1
then
	echo "Usage: ./scripts/update_version.sh VERSION"

	exit ${EXIT_FAILURE}
fi

VERSION=$1

sed "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" -i ./Cargo.toml
sed "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" -i ./fuzz/Cargo.toml
sed "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" -i ./python/Cargo.toml

exit ${EXIT_SUCCESS}
