#!/usr/bin/env python3

# Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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
import shutil
import tempfile

import pytest

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

        offset = data_stream.get_offset()
        assert offset == 8

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

        # Test read error when the backing image file is truncated.
        if os.name == "posix":
            with tempfile.TemporaryDirectory() as temporary_directory:
                image_path = os.path.join(temporary_directory, "ext2.qcow")
                shutil.copyfile(self.get_test_data_path("qcow/ext2.qcow"), image_path)

                resolver = vfs.VfsResolver()
                os_location = vfs.VfsLocation.new_base_from_string(
                    vfs.VfsType.OS, image_path
                )
                qcow_location = os_location.new_with_layer_from_string(
                    vfs.VfsType.QCOW, "/qcow1"
                )
                qcow_stream = resolver.get_data_stream_by_location_and_name(
                    qcow_location, None
                )
                assert qcow_stream is not None

                with open(image_path, "wb"):
                    pass

                with pytest.raises(RuntimeError):
                    _ = qcow_stream.read(4096)

    def test_seek(self):
        data_stream = self.get_data_stream("/testdir1/testfile1")

        assert data_stream is not None

        offset = data_stream.seek(8, whence=os.SEEK_SET)
        assert offset == 8

        offset = data_stream.seek(-4, whence=os.SEEK_CUR)
        assert offset == 4

        offset = data_stream.seek(-1, whence=os.SEEK_END)
        assert offset == 8

        with pytest.raises(ValueError):
            _ = data_stream.seek(-1, whence=os.SEEK_SET)

        with pytest.raises(ValueError):
            _ = data_stream.seek(0, whence=99)

        with pytest.raises(RuntimeError):
            _ = data_stream.seek(-100, whence=os.SEEK_CUR)

        with pytest.raises(RuntimeError):
            _ = data_stream.seek(-100, whence=os.SEEK_END)


class TestExtendedAttribute(TestClass):
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

    def test_get_data_stream(self):
        extended_attribute = self.get_extended_attribute("/testdir1/testfile1")

        assert extended_attribute is not None

        data_stream = extended_attribute.get_data_stream()
        assert data_stream is not None
        assert data_stream.get_size() > 0

        data = data_stream.read(data_stream.get_size())
        assert len(data) == data_stream.get_size()


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

        directory_entry = self.get_file_entry("/testdir1")
        assert directory_entry is not None
        assert directory_entry.file_type == vfs.VfsFileType.DIRECTORY

        block_device_entry = self.get_file_entry("/testdir1/blockdev1")
        assert block_device_entry is not None
        assert block_device_entry.file_type == vfs.VfsFileType.BLOCK_DEVICE
        assert block_device_entry.device_identifier == 6201

        character_device_entry = self.get_file_entry("/testdir1/chardev1")
        assert character_device_entry is not None
        assert character_device_entry.file_type == vfs.VfsFileType.CHARACTER_DEVICE
        assert character_device_entry.device_identifier == 3396

        named_pipe_entry = self.get_file_entry("/testdir1/pipe1")
        assert named_pipe_entry is not None
        assert named_pipe_entry.file_type == vfs.VfsFileType.NAMED_PIPE

        symbolic_link_entry = self.get_file_entry("/file_symboliclink1")
        assert symbolic_link_entry is not None
        assert symbolic_link_entry.file_type == vfs.VfsFileType.SYMBOLIC_LINK
        assert (
            symbolic_link_entry.symbolic_link_target.to_string()
            == "/mnt/keramics/testdir1/testfile1"
        )

        root_entry = self.get_file_entry("/")
        assert root_entry is not None
        assert root_entry.name is None

        # Test getters that return None for timestamps on a FAT root directory.
        resolver = vfs.VfsResolver()
        os_path_string = self.get_test_data_path("fat/fat12.raw")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        fat_root_location = os_location.new_with_layer_from_string(vfs.VfsType.FAT, "/")
        fat_root_entry = resolver.get_file_entry_by_location(fat_root_location)
        assert fat_root_entry is not None
        assert fat_root_entry.access_time is None
        assert fat_root_entry.change_time is None
        assert fat_root_entry.creation_time is None
        assert fat_root_entry.modification_time is None
        assert fat_root_entry.name is None

    def test_get_data_stream(self):
        file_entry = self.get_file_entry("/testdir1/testfile1")

        assert file_entry is not None

        data_stream = file_entry.get_data_stream()

        assert data_stream is not None

        directory_entry = self.get_file_entry("/testdir1")

        assert directory_entry is not None
        assert directory_entry.get_data_stream() is None

        if os.name == "posix" and os.geteuid() != 0:
            with tempfile.TemporaryDirectory() as temporary_directory:
                unreadable_file = os.path.join(temporary_directory, "unreadable.txt")
                with open(unreadable_file, "wb") as file_object:
                    file_object.write(b"test")
                os.chmod(unreadable_file, 0o000)
                try:
                    resolver = vfs.VfsResolver()
                    os_location = vfs.VfsLocation.new_base_from_string(
                        vfs.VfsType.OS, unreadable_file
                    )
                    unreadable_entry = resolver.get_file_entry_by_location(os_location)
                    assert unreadable_entry is not None
                    with pytest.raises(RuntimeError):
                        _ = unreadable_entry.get_data_stream()
                finally:
                    os.chmod(unreadable_file, 0o600)

    def test_get_number_of_extended_attributes(self):
        directory_entry = self.get_file_entry("/testdir1")

        assert directory_entry is not None
        assert directory_entry.get_number_of_extended_attributes() == 1

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

        if os.name == "posix" and os.geteuid() != 0:
            with tempfile.TemporaryDirectory() as temporary_directory:
                unreadable_directory = os.path.join(temporary_directory, "unreadable")
                os.mkdir(unreadable_directory)
                os.chmod(unreadable_directory, 0o000)
                try:
                    resolver = vfs.VfsResolver()
                    os_location = vfs.VfsLocation.new_base_from_string(
                        vfs.VfsType.OS, unreadable_directory
                    )
                    unreadable_entry = resolver.get_file_entry_by_location(os_location)
                    assert unreadable_entry is not None
                    with pytest.raises(RuntimeError):
                        _ = unreadable_entry.get_number_of_sub_file_entries()
                finally:
                    os.chmod(unreadable_directory, 0o700)

    def test_get_sub_file_entry_by_index(self):
        file_entry = self.get_file_entry("/testdir1")

        assert file_entry is not None

        sub_file_entry = file_entry.get_sub_file_entry_by_index(1)
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

        resolver = vfs.VfsResolver()
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, self.get_test_data_path("directory")
        )
        os_file_system = resolver.open_file_system(os_location)
        invalid_path = vfs.VfsPath.from_string("\0")
        with pytest.raises(RuntimeError):
            _ = os_file_system.file_entry_exists(invalid_path)

    def test_get_file_entry_by_path(self):
        file_system = self.get_file_system()

        assert file_system is not None

        path = vfs.VfsPath.from_string("/testdir1/testfile1")
        file_entry = file_system.get_file_entry_by_path(path)
        assert file_entry is not None

        path = vfs.VfsPath.from_string("/testdir1/bogus")
        file_entry = file_system.get_file_entry_by_path(path)
        assert file_entry is None

        resolver = vfs.VfsResolver()
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, self.get_test_data_path("directory")
        )
        os_file_system = resolver.open_file_system(os_location)
        invalid_path = vfs.VfsPath.from_string("\0")
        with pytest.raises(RuntimeError):
            _ = os_file_system.get_file_entry_by_path(invalid_path)

    def test_get_root_file_entry(self):
        file_system = self.get_file_system()

        assert file_system is not None

        root_file_entry = file_system.get_root_file_entry()
        assert root_file_entry is not None


class TestFileType(TestClass):
    def test_hash(self):
        file_types = [
            vfs.VfsFileType.BLOCK_DEVICE,
            vfs.VfsFileType.CHARACTER_DEVICE,
            vfs.VfsFileType.DEVICE,
            vfs.VfsFileType.DIRECTORY,
            vfs.VfsFileType.FILE,
            vfs.VfsFileType.NAMED_PIPE,
            vfs.VfsFileType.SOCKET,
            vfs.VfsFileType.SYMBOLIC_LINK,
            vfs.VfsFileType.WHITEOUT,
        ]
        for file_type in file_types:
            assert isinstance(hash(file_type), int)
        assert vfs.VfsFileType.FILE == vfs.VfsFileType.FILE
        assert vfs.VfsFileType.FILE != vfs.VfsFileType.DIRECTORY


class TestLocation(TestClass):
    def test_new_base_from_string(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )

        assert os_location is not None

        for vfs_type in [
            vfs.VfsType.APM,
            vfs.VfsType.EWF,
            vfs.VfsType.EXT,
            vfs.VfsType.FAKE,
            vfs.VfsType.FAT,
            vfs.VfsType.GPT,
            vfs.VfsType.MBR,
            vfs.VfsType.OS,
            vfs.VfsType.QCOW,
            vfs.VfsType.SPARSE_IMAGE,
            vfs.VfsType.UDIF,
            vfs.VfsType.VHD,
            vfs.VfsType.VHDX,
        ]:
            location = vfs.VfsLocation.new_base_from_string(vfs_type, "/")
            assert location is not None
            assert location.parent is None

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

        assert os_location.parent is None
        assert os_location.path is not None
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
        assert file_name.to_string() == "ext2.qcow2"

        root_path = vfs.VfsPath.from_string("/")
        assert root_path.get_file_name() is None

    def test_is_relative(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path.is_relative() is False

        relative_path = vfs.VfsPath.from_string("qcow/ext2.qcow2")
        assert relative_path.is_relative() is True

    def test_is_root(self):
        os_path_string = self.get_test_data_path("qcow/ext2.qcow2")
        os_path = vfs.VfsPath.from_string(os_path_string)

        assert os_path.is_root() is False

        root_path = vfs.VfsPath.from_string("/")
        assert root_path.is_root() is True

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

        data_stream_name = vfs.VfsPathComponent.from_string("bogus")
        data_stream = resolver.get_data_stream_by_location_and_name(
            ext_location, data_stream_name
        )

        assert data_stream is None

        ext_location = qcow_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/bogus"
        )
        data_stream = resolver.get_data_stream_by_location_and_name(ext_location, None)

        assert data_stream is None

        missing_os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, self.get_test_data_path("nonexistent.qcow2")
        )
        invalid_qcow_location = missing_os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        with pytest.raises(RuntimeError):
            _ = resolver.get_data_stream_by_location_and_name(
                invalid_qcow_location, None
            )

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

        missing_os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, self.get_test_data_path("nonexistent.qcow2")
        )
        invalid_qcow_location = missing_os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        with pytest.raises(RuntimeError):
            _ = resolver.get_file_entry_by_location(invalid_qcow_location)

        # Test resolving across additional supported VFS layer types.
        for relative_path, vfs_type, layer_path in [
            ("apm/apm.dmg", vfs.VfsType.APM, "/apm1"),
            ("ewf/ext2.E01", vfs.VfsType.EWF, "/ewf1"),
            ("fat/fat12.raw", vfs.VfsType.FAT, "/testdir1/testfile1"),
            ("gpt/gpt.raw", vfs.VfsType.GPT, "/gpt1"),
            ("mbr/mbr.raw", vfs.VfsType.MBR, "/mbr1"),
            (
                "sparseimage/hfsplus.sparseimage",
                vfs.VfsType.SPARSE_IMAGE,
                "/sparseimage1",
            ),
            ("udif/hfsplus_zlib.dmg", vfs.VfsType.UDIF, "/udif1"),
            ("vhd/ext2.vhd", vfs.VfsType.VHD, "/vhd1"),
            ("vhdx/ext2.vhdx", vfs.VfsType.VHDX, "/vhdx1"),
        ]:
            base_location = vfs.VfsLocation.new_base_from_string(
                vfs.VfsType.OS, self.get_test_data_path(relative_path)
            )
            layer_location = base_location.new_with_layer_from_string(
                vfs_type, layer_path
            )
            layer_file_entry = resolver.get_file_entry_by_location(layer_location)
            assert layer_file_entry is not None

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

        missing_os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, self.get_test_data_path("nonexistent.qcow2")
        )
        invalid_qcow_location = missing_os_location.new_with_layer_from_string(
            vfs.VfsType.QCOW, "/qcow1"
        )
        with pytest.raises(RuntimeError):
            _ = resolver.open_file_system(invalid_qcow_location)


class TestType(TestClass):
    def test_hash(self):
        vfs_types = [
            vfs.VfsType.APM,
            vfs.VfsType.EWF,
            vfs.VfsType.EXT,
            vfs.VfsType.FAKE,
            vfs.VfsType.FAT,
            vfs.VfsType.GPT,
            vfs.VfsType.MBR,
            vfs.VfsType.OS,
            vfs.VfsType.QCOW,
            vfs.VfsType.SPARSE_IMAGE,
            vfs.VfsType.UDIF,
            vfs.VfsType.VHD,
            vfs.VfsType.VHDX,
        ]
        for vfs_type in vfs_types:
            assert isinstance(hash(vfs_type), int)
        assert vfs.VfsType.APM == vfs.VfsType.APM
        assert vfs.VfsType.APM != vfs.VfsType.EXT
