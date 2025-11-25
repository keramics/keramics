#!/usr/bin/env python3

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


class TestClass:
    def get_test_data_path(self, path):
        return os.path.join(
            os.path.dirname(os.path.dirname(os.path.dirname(__file__))),
            "test_data",
            path,
        )


class TestDataStream(TestClass):
    def get_data_stream(self, path):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(vfs.VfsType.EXT, path)
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

        data = data_stream.read(8)
        assert len(data) == 8
        assert data == b"Keramics"

        data = data_stream.read(8)
        assert len(data) == 1
        assert data == b"\n"

    def test_seek(self):
        data_stream = self.get_data_stream("/testdir1/testfile1")

        assert data_stream is not None

        offset = data_stream.seek(8, whence=os.SEEK_SET)
        assert offset == 8

        with pytest.raises(ValueError):
            _ = data_stream.seek(-1, whence=os.SEEK_SET)

        with pytest.raises(ValueError):
            _ = data_stream.seek(0, whence=99)


class TestDataStream(TestClass):
    def get_extended_attribute(self, path):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(vfs.VfsType.EXT, path)

        file_entry = resolver.get_file_entry_by_location(ext_location)
        name = vfs.VfsPathComponent.from_string("security.selinux")
        return file_entry.get_extended_attribute_by_name(name)

    def test_getters(self):
        extended_attribute = self.get_extended_attribute("/testdir1/testfile1")

        assert extended_attribute is not None
        assert extended_attribute.name.to_string() == "security.selinux"


class TestFileEntry(TestClass):
    def get_file_entry(self, path):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(vfs.VfsType.EXT, path)
        return resolver.get_file_entry_by_location(ext_location)

    def test_getters(self):
        file_entry = self.get_file_entry("/testdir1/testfile1")

        assert file_entry is not None
        assert file_entry.access_time.timestamp == 1735977482
        assert file_entry.change_time.timestamp == 1735977481
        assert file_entry.creation_time is None
        assert file_entry.device_identifier is None
        assert file_entry.file_mode == 0o100644
        assert file_entry.file_type == vfs.VfsFileType.FILE
        # TODO: add deletion_time
        assert file_entry.group_identifier == 1000
        assert file_entry.inode_number == 14
        assert file_entry.modification_time.timestamp == 1735977481
        assert file_entry.name.to_string() == "testfile1"
        assert file_entry.number_of_links == 2
        assert file_entry.owner_identifier == 1000
        assert file_entry.size == 9
        assert file_entry.symbolic_link_target is None

    def test_get_data_stream(self):
        file_entry = self.get_file_entry("/testdir1/testfile1")

        assert file_entry is not None

        data_stream = file_entry.get_data_stream()

        assert data_stream is not None

    def test_get_number_of_extended_attributes(self):
        file_entry = self.get_file_entry("/testdir1")

        file_entry = self.get_file_entry("/testdir1/testfile1")

        number_of_extended_attributes = file_entry.get_number_of_extended_attributes()
        assert number_of_extended_attributes == 1

    def test_get_extended_attribute_by_index(self):
        file_entry = self.get_file_entry("/testdir1/testfile1")

        assert file_entry is not None

        extended_attribute = file_entry.get_extended_attribute_by_index(0)
        assert extended_attribute is not None
        assert extended_attribute.name.to_string() == "security.selinux"

        with pytest.raises(RuntimeError):
            _ = file_entry.get_extended_attribute_by_index(99)

    def test_get_extended_attribute_by_name(self):
        file_entry = self.get_file_entry("/testdir1/testfile1")

        assert file_entry is not None

        name = vfs.VfsPathComponent.from_string("security.selinux")
        extended_attribute = file_entry.get_extended_attribute_by_name(name)
        assert extended_attribute is not None
        assert extended_attribute.name.to_string() == "security.selinux"

        name = vfs.VfsPathComponent.from_string("bogus")
        extended_attribute = file_entry.get_extended_attribute_by_name(name)
        assert extended_attribute is None

    def test_get_number_of_sub_file_entries(self):
        file_entry = self.get_file_entry("/testdir1")

        assert file_entry is not None

        number_of_sub_file_entries = file_entry.get_number_of_sub_file_entries()
        assert number_of_sub_file_entries == 10

        file_entry = self.get_file_entry("/testdir1/testfile1")

        number_of_sub_file_entries = file_entry.get_number_of_sub_file_entries()
        assert number_of_sub_file_entries == 0

    def test_get_sub_file_entry_by_index(self):
        file_entry = self.get_file_entry("/testdir1")

        assert file_entry is not None

        sub_file_entry = file_entry.get_sub_file_entry_by_index(0)
        assert sub_file_entry is not None
        assert sub_file_entry.name.to_string() == "TestFile2"

        with pytest.raises(RuntimeError):
            _ = file_entry.get_sub_file_entry_by_index(99)

    def test_sub_file_entries(self):
        class VfsFileEntriesIterator:
            def __init__(self, file_entry):
                self._file_entry = file_entry
                self._sub_file_entry_index = 0
                self._number_of_sub_file_entries = (
                    file_entry.get_number_of_sub_file_entries()
                )

            def __iter__(self):
                return self

            def __next__(self):
                if self._sub_file_entry_index >= self._number_of_sub_file_entries:
                    raise StopIteration

                sub_file_entry = self._file_entry.get_sub_file_entry_by_index(
                    self._sub_file_entry_index
                )
                self._sub_file_entry_index += 1
                return sub_file_entry

        file_entry = self.get_file_entry("/testdir1")

        assert file_entry is not None

        file_names = sorted(
            [
                sub_file_entry.name.to_string()
                for sub_file_entry in VfsFileEntriesIterator(file_entry)
            ]
        )
        assert file_names == sorted(
            [
                "blockdev1",
                "chardev1",
                "initial_sparse1",
                "pipe1",
                "testfile1",
                "TestFile2",
                "trailing_sparse1",
                "uninitialized1",
                "xattr1",
                "xattr2",
            ]
        )


class TestFileSystem(TestClass):
    def get_file_system(self):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/testdir1/testfile1"
        )
        return resolver.open_file_system(ext_location)

    def test_file_entry_exists(self):
        file_system = self.get_file_system()

        assert file_system is not None

        path = vfs.VfsPath.from_string("/testdir1/testfile1")
        assert file_system.file_entry_exists(path)

        path = vfs.VfsPath.from_string("/testdir1/bogus")
        assert not file_system.file_entry_exists(path)

    def test_get_file_entry_by_path(self):
        file_system = self.get_file_system()

        assert file_system is not None

        path = vfs.VfsPath.from_string("/testdir1/testfile1")
        file_entry = file_system.get_file_entry_by_path(path)
        assert file_entry is not None

        path = vfs.VfsPath.from_string("/testdir1/bogus")
        file_entry = file_system.get_file_entry_by_path(path)
        assert file_entry is None

    def test_get_root_file_entry(self):
        file_system = self.get_file_system()

        assert file_system is not None

        root_file_entry = file_system.get_root_file_entry()
        assert root_file_entry is not None


class TestFileType(TestClass):
    def test_hash(self):
        hash(vfs.VfsFileType.FILE)


class TestLocation(TestClass):
    def test_new_base_from_string(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )

        assert os_location is not None

    def test_new_with_layer_from_string(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )

        assert qcow_location is not None

    def test_new_with_parent(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )

        assert qcow_location is not None

        test_location = qcow_location.new_with_parent(vfs.VfsPath.from_string("/qcow1"))

        assert test_location is not None

    def test_getters(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )

        assert qcow_location.path is not None
        assert qcow_location.parent is not None

    def test_to_string(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )

        location_string = qcow_location.to_string()
        assert location_string.startswith("OS: ")
        assert location_string.endswith("/test_data/qcow/ext2.qcow2\nQCOW: /qcow1\n")


class TestPath(TestClass):
    def test_from_string(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path is not None

    def test_new_with_join(self):
        os_path_string = self.get_test_data_path("qcow")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path is not None

        path = vfs.VfsPath.from_string("ext2.qcow2")
        test_path = os_path.new_with_join(path)
        assert test_path is not None

    def test_new_with_join_path_components(self):
        os_path_string = self.get_test_data_path("qcow")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path is not None

        path_component_strings = [vfs.VfsPathComponent.from_string("ext2.qcow2")]
        test_path = os_path.new_with_join_path_components(path_component_strings)
        assert test_path is not None

    def test_new_with_parent_directory(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        test_path = os_path.new_with_parent_directory()
        assert test_path is not None

    def test_get_file_name(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        file_name = os_path.get_file_name()
        assert file_name is not None

    def test_is_relative(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path.is_relative() is False

    def test_is_root(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path.is_root() is False

    def test_to_string(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        path_string = os_path.to_string()
        assert path_string.endswith("/test_data/qcow/ext2.qcow2")


class TestPathComponent(TestClass):
    def test_from_string(self):
        path_component = vfs.VfsPathComponent.from_string("ext2.qcow2")

        assert path_component is not None

    def test_to_string(self):
        path_component = vfs.VfsPathComponent.from_string("ext2.qcow2")

        assert path_component is not None
        assert path_component.to_string() == "ext2.qcow2"


class TestResolver(TestClass):
    def test_get_data_stream_by_location_and_name(self):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/testdir1/testfile1"
        )
        data_stream = resolver.get_data_stream_by_location_and_name(ext_location, None)

        assert data_stream is not None

        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/bogus"
        )
        data_stream = resolver.get_data_stream_by_location_and_name(ext_location, None)

        assert data_stream is None

    def test_get_file_entry_by_location(self):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/testdir1/testfile1"
        )
        file_entry = resolver.get_file_entry_by_location(ext_location)

        assert file_entry is not None

        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/bogus"
        )
        file_entry = resolver.get_file_entry_by_location(ext_location)

        assert file_entry is None

    def test_open_file_system(self):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        qcow_location = os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/testdir1/testfile1"
        )
        file_system = resolver.open_file_system(ext_location)

        assert file_system is not None


class TestType(TestClass):
    def test_hash(self):
        hash(vfs.VfsType.APM)
