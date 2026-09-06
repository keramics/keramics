# Script to generate Keramics BDE test files on Windows.
# Requires Windows 10 / Server 2016 or later for advanced storage/case-sensitivity features.

function CreateTestFileEntries {
    param (
        [Parameter(Mandatory=$true)]
        [string]$DriveLetter
    )
    # Create an empty file
    New-Item -Path "${DriveLetter}:\emptyfile" -ItemType File -Force

    # Create a directory
    New-Item -Path "${DriveLetter}:\testdir1" -ItemType Directory -Force

    # Create a file that can be stored as inline data
    "Keramics" | Out-File -FilePath "${DriveLetter}:\testdir1\testfile1" -Encoding ascii

    # Create a file that cannot be stored as inline data
    Copy-Item -Path "LICENSE" -Destination "${DriveLetter}:\testdir1\TestFile2" -ErrorAction SilentlyContinue

    # Create a file with a long filename
    New-Item -Path "${DriveLetter}:\My long, very long file name, so very long" -ItemType File -Force

    # Create a symbolic link to a file
    New-Item -ItemType SymbolicLink -Path "${DriveLetter}:\file_symboliclink1" -Target "${DriveLetter}:\testdir1\testfile1" -Force

    # Create a junction (hard link to a directory)
    New-Item -ItemType Junction -Path "${DriveLetter}:\directory_junction1" -Target "${DriveLetter}:\testdir1" -Force

    # Create a symbolic link to a directory
    New-Item -ItemType SymbolicLink -Path "${DriveLetter}:\directory_symboliclink1" -Target "${DriveLetter}:\testdir1" -Force

    # Create a file with an alternative data stream (ADS)
    New-Item -Path "${DriveLetter}:\file_ads1" -ItemType File -Force
    Set-Content -Path "${DriveLetter}:\file_ads1" -Stream "myads" -Value "My file ADS"

    # Create a directory with an alternative data stream (ADS)
    New-Item -Path "${DriveLetter}:\directory_ads1" -ItemType Directory -Force
    Set-Content -Path "${DriveLetter}:\directory_ads1" -Stream "myads" -Value "My directory ADS"

    # Create a file with valid data size set
    Copy-Item -Path "LICENSE" -Destination "${DriveLetter}:\testdir1\file_valid_data_size1" -ErrorAction SilentlyContinue
    fsutil file setValidData "${DriveLetter}:\testdir1\file_valid_data_size1" 18652

    # Create a file with short name set
    "My short file" | Out-File -FilePath "${DriveLetter}:\testdir1\file_short_name1" -Encoding ascii
    fsutil file setShortName "${DriveLetter}:\testdir1\file_short_name1" short1

    # Create the parent and child directories
    New-Item -Path "${DriveLetter}:\testdir2\normal" -ItemType Directory -Force
    New-Item -Path "${DriveLetter}:\testdir2\sensitive" -ItemType Directory -Force

    # Enable case-sensitivity on the specific folder
    fsutil file setCaseSensitiveInfo "${DriveLetter}:\testdir2\sensitive" enable

    # Write files to the normal (case-insensitive) directory
    # Note that the second command will OVERWRITE the first file because NTFS ignores case by default here.
    "My second file" | Out-File -FilePath "${DriveLetter}:\testdir2\normal\testfile1" -Encoding ascii
    "My third file"  | Out-File -FilePath "${DriveLetter}:\testdir2\normal\TestFile1" -Encoding ascii

    # Write files to the case-sensitive directory
    # Note that the second command will OVERWRITE the first file because NTFS ignores case by default here.
    "My second file" | Out-File -FilePath "${DriveLetter}:\testdir2\sensitive\testfile1" -Encoding ascii
    "My third file"  | Out-File -FilePath "${DriveLetter}:\testdir2\sensitive\TestFile1" -Encoding ascii
}

# Alternative for New-VHD, given it is not always available.
function CreateAndMountVhd {
    param (
        [Parameter(Mandatory=$true)]
        [string]$ImageFullPath,

        [Parameter(Mandatory=$true)]
        [int]$ImageSize

        [Parameter(Mandatory=$true)]
        [string]$ImageType
    )
    $DiskpartScript = Join-Path $env:TEMP "CreateVHD.diskpart"

@"
create vdisk file="${ImageFullPath}" maximum=${ImageSize} type=${ImageType}
select vdisk file="${ImageFullPath}"
attach vdisk
convert mbr
create partition primary
format fs=ntfs label="TestVolume" unit=4096 quick
assign letter=x
"@ | Out-File -FilePath ${DiskpartScript} -Encoding ascii

    diskpart /s ${DiskpartScript}

    Remove-Item ${DiskpartScript} -Force
}

# Alternative for Dismount-VHD, given it is not always available.
function UnmountVhd {
    param (
        [Parameter(Mandatory=$true)]
        [string]$ImageFullPath
    )
    $DiskpartScript = Join-Path $env:TEMP "UnmountVHD.diskpart"

@"
select vdisk file="${ImageFullPath}"
detach vdisk
"@ | Out-File -FilePath ${DiskpartScript} -Encoding ascii

    diskpart /s ${DiskpartScript}

    Remove-Item ${DiskpartScript} -Force
}

$ErrorActionPreference = "Stop"

if (-not (Test-Path "test_data")) {
   New-Item -ItemType Directory -Path "test_data" | Out-Null
}
if (-not (Test-Path "test_data\bde")) {
    New-Item -ItemType Directory -Path "test_data\bde" | Out-Null
}

$UnitSize = 4096
$ImageSize = 64
$DriveLetter = "X"

# Create an AES-128-CBC encrypted BDE image.
$ImageName = "bde_aes128_used_space.vhd"
$ImageFullPath = "${Pwd}\test_data\bde\${ImageName}"

if (Test-Path $ImageFullPath) {
    Remove-Item -Force $ImageFullPath
}

Write-Host "Creating: ${ImageName}"
CreateAndMountVhd -ImageFullPath ${ImageFullPath} -ImageSize ${ImageSize} -ImageType "expandable"
CreateTestFileEntries -DriveLetter ${DriveLetter}

$SecurePassword = ConvertTo-SecureString "KeRaMiCs" -AsPlainText -Force
Enable-BitLocker -MountPoint "${DriveLetter}:" -EncryptionMethod Aes128 -PasswordProtector -Password ${SecurePassword} -UsedSpaceOnly | Out-Null

UnmountVhd -ImageFullPath ${ImageFullPath}
