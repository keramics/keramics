#!/usr/bin/env python

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

import pytest

from pykeramics import datetime
from pykeramics import vfs


def test_get_file_entry_by_location() -> None:
    resolver = vfs.VfsResolver()

    os_location = vfs.VfsLocation(
        vfs.VfsType.OS, vfs.VfsPath(vfs.VfsType.OS, "../test_data/qcow/ext2.qcow2")
    )
    qcow_location = os_location.new_with_layer(
        vfs.VfsType.QCOW, vfs.VfsPath(vfs.VfsType.QCOW, "/qcow1")
    )
    ext_location = qcow_location.new_with_layer(
        vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, "/testdir1/testfile1")
    )
    file_entry = resolver.get_file_entry_by_location(ext_location)

    assert file_entry is not None
    assert file_entry.name.to_string() == "testfile1"
    assert file_entry.access_time.timestamp == 1735977482
    assert file_entry.change_time.timestamp == 1735977481
    assert file_entry.creation_time is None
    assert file_entry.modification_time.timestamp == 1735977481

    ext_location = qcow_location.new_with_layer(
        vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, "/bogus")
    )
    file_entry = resolver.get_file_entry_by_location(ext_location)

    assert file_entry is None


def test_open_file_system() -> None:
    resolver = vfs.VfsResolver()

    os_location = vfs.VfsLocation(
        vfs.VfsType.OS, vfs.VfsPath(vfs.VfsType.OS, "../test_data/qcow/ext2.qcow2")
    )
    qcow_location = os_location.new_with_layer(
        vfs.VfsType.QCOW, vfs.VfsPath(vfs.VfsType.QCOW, "/qcow1")
    )
    ext_location = qcow_location.new_with_layer(
        vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, "/testdir1/testfile1")
    )
    file_system = resolver.open_file_system(ext_location)

    assert file_system is not None
