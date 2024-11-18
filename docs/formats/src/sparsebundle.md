# Mac OS sparse bundle (.sparsebundle) format

The Mac OS sparse bundle (.sparsebundle) format is one of the disk image formats
supported natively by Mac OS.

The sparse bundle disk image was introduced in Mac OS X 10.5.

## Overview

A sparse bundle consists of a directory (bundle) with the .sparsbundle suffix
containing:

* "Info.bckup" file
* "Info.plist" file
* "token" file
* "bands" directory containing the band files

### Characteristics

| Characteristics | Description
| --- | ---
| Byte order | N/A
| Date and time values | N/A
| Character strings | N/A

## Info.plist and Info.bckup files

The Info.plist and its backup (Info.bckup) contain a [XML plist](plist.md).

This plist is also referred to as "Information Property List" and contains a
single dictionary with the following key-value pairs.

| Identifier | Value | Description
| --- | --- | ---
| CFBundleInfoDictionaryVersion | "6.0" | The information property list format version
| band-size | | The maximum size of a band file in bytes
| bundle-backingstore-version | 1 | Unknown
| diskimage-bundle-type | "com.apple.diskimage.sparsebundle" | The bundle type
| size | | The media size in bytes

```
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
        <key>CFBundleInfoDictionaryVersion</key>
        <string>6.0</string>
        <key>band-size</key>
        <integer>8388608</integer>
        <key>bundle-backingstore-version</key>
        <integer>1</integer>
        <key>diskimage-bundle-type</key>
        <string>com.apple.diskimage.sparsebundle</string>
        <key>size</key>
        <integer>4194304</integer>
</dict>
</plist>
```

## Token file

The token file is empty.

## Bands directory

The bands directory contains files containing the actual data of the bands. The
files are named using a hexadecimal naming scheme where "0" is the 1st band,
"a" the 10th, "f" the 15th, "10" the 16th, etc.
