@echo off

rem Script to generate Keramics BDE test files on Windows.
rem Requires Windows 7 or later with TPM (for manage-bde).

if not exist "test_data" (
    mkdir "test_data"
)
if not exist "test_data\bde" (
    mkdir "test_data\bde"
)

rem Create a dynamic-size VHD image with an AES 128-bit BDE encrypted volume with a password
set unitsize=4096
set imagename=bde_aes128.vhd
set imagesize=64

del /f %cd%\test_data\bde\%imagename%

echo Creating: %imagename%

echo create vdisk file=%cd%\test_data\bde\%imagename% maximum=%imagesize% type=expandable > CreateVHD.diskpart
echo select vdisk file=%cd%\test_data\bde\%imagename% >> CreateVHD.diskpart
echo attach vdisk >> CreateVHD.diskpart
echo convert mbr >> CreateVHD.diskpart
echo create partition primary >> CreateVHD.diskpart

echo format fs=ntfs label="TestVolume" unit=%unitsize% quick >> CreateVHD.diskpart

echo assign letter=x >> CreateVHD.diskpart

call :run_diskpart CreateVHD.diskpart

call :create_test_file_entries x

rem This will ask for a password
manage-bde -On x: -DiscoveryVolumeType "[none]" -EncryptionMethod aes128 -Password -Synchronous

echo select vdisk file=%cd%\test_data\bde\%imagename% > UnmountVHD.diskpart
echo detach vdisk >> UnmountVHD.diskpart

call :run_diskpart UnmountVHD.diskpart

exit /b 0

rem Creates test file entries
:create_test_file_entries
SETLOCAL
SET driveletter=%1

rem Create an empty file
type nul >> %driveletter%:\emptyfile

rem Create a directory
mkdir %driveletter%:\testdir1

rem Create a file that can be stored as inline data
echo Keramics > %driveletter%:\testdir1\testfile1

rem Create a file that cannot be stored as inline data
copy LICENSE %driveletter%:\testdir1\TestFile2

rem Create a file with a long filename
type nul >> "%driveletter%:\My long, very long file name, so very long"

rem Create a symbolic link to a file
mklink %driveletter%:\file_symboliclink1 %driveletter%:\testdir1\testfile1

rem Create a junction (hard link to a directory)
mklink /J %driveletter%:\directory_junction1 %driveletter%:\testdir1

rem Create a symbolic link to a directory
mklink /D %driveletter%:\directory_symboliclink1 %driveletter%:\testdir1

rem Create a file with an alternative data stream (ADS)
type nul >> %driveletter%:\file_ads1
echo My file ADS > %driveletter%:\file_ads1:myads

rem Create a directory with an alternative data stream (ADS)
mkdir %driveletter%:\directory_ads1
echo My directory ADS > %driveletter%:\directory_ads1:myads

rem Create a file with valid data size set
copy LICENSE %driveletter%:\testdir1\file_valid_data_size1
fsutil file setValidData %driveletter%:\testdir1\file_valid_data_size1 18652

rem Create a file with short name set
echo My short file > %driveletter%:\testdir1\file_short_name1
fsutil file setShortName %driveletter%:\testdir1\file_short_name1 short1

rem Create a file with a sparse data run
copy LICENSE %driveletter%:\testdir1\file_sparse1
fsutil sparse setflag %driveletter%:\testdir1\file_sparse1
fsutil sparse setRange %driveletter%:\testdir1\file_sparse1 0 18000

rem Create a case-sensitive directory
rem This requires Microsoft-Windows-Subsystem-Linux to be enabled
rem Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
mkdir %driveletter%:\testdir2\normal
mkdir %driveletter%:\testdir2\sensitive
fsutil file setCaseSensitiveInfo %driveletter%:\testdir2\sensitive enable

echo My second file > %driveletter%:\testdir2\normal\testfile1
echo My second file > %driveletter%:\testdir2\sensitive\testfile1

echo My third file > %driveletter%:\testdir2\normal\TestFile1
echo My third file > %driveletter%:\testdir2\sensitive\TestFile1

ENDLOCAL
exit /b 0

rem Runs diskpart with a script
rem Note that diskpart requires Administrator privileges to run
:run_diskpart
SETLOCAL
set diskpartscript=%1

rem Note that diskpart requires Administrator privileges to run
diskpart /s %diskpartscript%

if %errorlevel% neq 0 (
    echo Failed to run: "diskpart /s %diskpartscript%"

    exit /b 1
)

del /q %diskpartscript%

rem Give the system a bit of time to adjust
timeout /t 1 > nul

ENDLOCAL
exit /b 0
