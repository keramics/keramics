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
import tempfile

from pykeramics import datetime
from pykeramics import vfs


class TestDateTime:
    def get_test_data_path(self, path):
        return os.path.join(
            os.path.dirname(os.path.dirname(os.path.dirname(__file__))),
            "test_data",
            path,
        )

    def test_module_classes(self):
        assert hasattr(datetime, "FatDate")
        assert hasattr(datetime, "FatTimeDate")
        assert hasattr(datetime, "FatTimeDate10Ms")
        assert hasattr(datetime, "Filetime")
        assert hasattr(datetime, "PosixTime32")
        assert hasattr(datetime, "PosixTime64Ns")

    def test_fat_date_times(self):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("fat/fat12.raw")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        fat_location = os_location.new_with_layer_from_string(
            vfs.VfsType.FAT, "/testdir1/testfile1"
        )
        file_entry = resolver.get_file_entry_by_location(fat_location)
        assert file_entry is not None

        access_time = file_entry.access_time
        assert isinstance(access_time, datetime.FatDate)
        assert access_time.timestamp == 144529920000

        modification_time = file_entry.modification_time
        assert isinstance(modification_time, datetime.FatTimeDate)
        assert modification_time.timestamp == 144536667000

        creation_time = file_entry.creation_time
        assert isinstance(creation_time, datetime.FatTimeDate10Ms)
        assert creation_time.timestamp == 144536667125

    def test_posix_time32(self):
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

        access_time = file_entry.access_time
        assert isinstance(access_time, datetime.PosixTime32)
        assert access_time.timestamp == 1735977482

    def test_posix_time64_ns(self):
        resolver = vfs.VfsResolver()

        os_path_string = self.get_test_data_path("ext/ext4.raw")
        os_location = vfs.VfsLocation.new_base_from_string(
            vfs.VfsType.OS, os_path_string
        )
        ext_location = os_location.new_with_layer_from_string(
            vfs.VfsType.EXT, "/testdir1/testfile1"
        )
        file_entry = resolver.get_file_entry_by_location(ext_location)
        assert file_entry is not None

        access_time = file_entry.access_time
        assert isinstance(access_time, datetime.PosixTime64Ns)
        assert access_time.timestamp == 1735977482
        assert access_time.fraction == 235516933

        change_time = file_entry.change_time
        assert isinstance(change_time, datetime.PosixTime64Ns)
        assert change_time.timestamp == 1735977482
        assert change_time.fraction == 237516930

        creation_time = file_entry.creation_time
        assert isinstance(creation_time, datetime.PosixTime64Ns)
        assert creation_time.timestamp == 1735977482
        assert creation_time.fraction == 235516933

        modification_time = file_entry.modification_time
        assert isinstance(modification_time, datetime.PosixTime64Ns)
        assert modification_time.timestamp == 1735977482
        assert modification_time.fraction == 235516933

    def test_os_date_times(self):
        resolver = vfs.VfsResolver()

        if os.name == "posix":
            with tempfile.NamedTemporaryFile(delete=False) as temp_file:
                temp_path = temp_file.name
            try:
                os.utime(temp_path, (0, 0))
                os_location = vfs.VfsLocation.new_base_from_string(
                    vfs.VfsType.OS, temp_path
                )
                file_entry = resolver.get_file_entry_by_location(os_location)
                assert file_entry is not None
                assert file_entry.access_time is None
                assert file_entry.modification_time is None
            finally:
                os.unlink(temp_path)
        elif os.name == "nt":
            os_path_string = self.get_test_data_path("directory/file.txt")
            os_location = vfs.VfsLocation.new_base_from_string(
                vfs.VfsType.OS, os_path_string
            )
            file_entry = resolver.get_file_entry_by_location(os_location)
            assert file_entry is not None
            assert isinstance(file_entry.access_time, datetime.Filetime)
            assert file_entry.access_time.timestamp > 0
