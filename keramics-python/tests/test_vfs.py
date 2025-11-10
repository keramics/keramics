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

import os

import pytest

from pykeramics import datetime
from pykeramics import vfs


class TestDataStream:
    def get_data_stream(self, path):
        resolver = vfs.VfsResolver()

        os_location = vfs.VfsLocation(
            vfs.VfsType.OS, vfs.VfsPath(vfs.VfsType.OS, "../test_data/qcow/ext2.qcow2")
        )
        qcow_location = os_location.new_with_layer(
            vfs.VfsType.QCOW, vfs.VfsPath(vfs.VfsType.QCOW, "/qcow1")
        )
        ext_location = qcow_location.new_with_layer(
            vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, path)
        )
        return resolver.get_data_stream_by_location_and_name(ext_location, None)

    def test_get_offset(self):
        data_stream = self.get_data_stream("/testdir1/testfile1")

        assert data_stream is not None

        offset = data_stream.seek(8, whence=os.SEEK_SET)

        ofset = data_stream.get_offset()
        assert ofset == 8

    def test_get_size(self):
        data_stream = self.get_data_stream("/testdir1/testfile1")

        assert data_stream is not None

        size = data_stream.get_size()
        assert size == 9

    def test_read(self):
        data_stream = self.get_data_stream("/testdir1/testfile1")

        assert data_stream is not None

        data = data_stream.read(size=8)
        assert data == b"Keramics"

        with pytest.raises(RuntimeError):
            _ = data_stream.read()

    def test_seek(self):
        data_stream = self.get_data_stream("/testdir1/testfile1")

        assert data_stream is not None

        offset = data_stream.seek(8, whence=os.SEEK_SET)
        assert offset == 8

        with pytest.raises(RuntimeError):
            _ = data_stream.seek(0, 99)


class TestFileEntry:
    def get_file_entry(self, path):
        resolver = vfs.VfsResolver()

        os_location = vfs.VfsLocation(
            vfs.VfsType.OS, vfs.VfsPath(vfs.VfsType.OS, "../test_data/qcow/ext2.qcow2")
        )
        qcow_location = os_location.new_with_layer(
            vfs.VfsType.QCOW, vfs.VfsPath(vfs.VfsType.QCOW, "/qcow1")
        )
        ext_location = qcow_location.new_with_layer(
            vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, path)
        )
        return resolver.get_file_entry_by_location(ext_location)

    def test_getters(self):
        file_entry = self.get_file_entry("/testdir1/testfile1")

        assert file_entry is not None
        assert file_entry.access_time.timestamp == 1735977482
        assert file_entry.change_time.timestamp == 1735977481
        assert file_entry.creation_time is None
        assert file_entry.file_type == vfs.VfsFileType.FILE
        # TODO: add deletion_time
        assert file_entry.modification_time.timestamp == 1735977481
        assert file_entry.name.to_string() == "testfile1"
        assert file_entry.size == 9
        assert file_entry.symbolic_link_target is None


class TestFileSystem:
    def get_file_system(self):
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
        return resolver.open_file_system(ext_location)

    def test_file_entry_exists(self):
        file_system = self.get_file_system()

        assert file_system is not None

        assert file_system.file_entry_exists(
            vfs.VfsPath(vfs.VfsType.EXT, "/testdir1/testfile1")
        )
        assert not file_system.file_entry_exists(
            vfs.VfsPath(vfs.VfsType.EXT, "/testdir1/bogus")
        )

    def test_get_file_entry_by_path(self):
        file_system = self.get_file_system()

        assert file_system is not None

        file_entry = file_system.get_file_entry_by_path(
            vfs.VfsPath(vfs.VfsType.EXT, "/testdir1/testfile1")
        )
        assert file_entry is not None

        file_entry = file_system.get_file_entry_by_path(
            vfs.VfsPath(vfs.VfsType.EXT, "/testdir1/bogus")
        )
        assert file_entry is None

    def test_get_root_file_entry(self):
        file_system = self.get_file_system()

        assert file_system is not None

        root_file_entry = file_system.get_root_file_entry()
        assert root_file_entry is not None


class TestResolver:
    def test_get_data_stream_by_location_and_name(self):
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
        data_stream = resolver.get_data_stream_by_location_and_name(ext_location, None)

        assert data_stream is not None

        ext_location = qcow_location.new_with_layer(
            vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, "/bogus")
        )
        data_stream = resolver.get_data_stream_by_location_and_name(ext_location, None)

        assert data_stream is None

    def test_get_file_entry_by_location(self):
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

        ext_location = qcow_location.new_with_layer(
            vfs.VfsType.EXT, vfs.VfsPath(vfs.VfsType.EXT, "/bogus")
        )
        file_entry = resolver.get_file_entry_by_location(ext_location)

        assert file_entry is None

    def test_open_file_system(self):
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
