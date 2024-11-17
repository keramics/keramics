# Property list (plist) format

The property list (plist) formats are used to store various kinds of data, for
example configuration data. The format is know to be used stand-alone as well
as embedded in other data formats.

## Overview

Known plist formats are:

* ASCII plist format
* Binary plist format
* XML plist format

TODO: What about other plist formats like JSON?

### Value types

| Type | Description
| --- | ---
| array | Collection of plist values without key
| boolean | Boolean value
| data | Binary data
| date | Date and time value
| dictionary | Collection of plist values with key
| integer | Signed integer value
| real | Floating-point value
| string | String value

## ASCII plist format

TODO: complete section

## Binary plist format

A binary plist file consists of:

* header
* object table
* offset table
* trailer

| Characteristics | Description
| --- | ---
| Byte order | big-endian
| Date and time values | Number of seconds since Jan 1, 2001 00:00:00 UTC
| Character strings | UTF-16 big-endian

### Binary plist header

The binary plist header (CFBinaryPlistHeader) is 8 bytes in size and consists
of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 6 | "bplist" | Signature
| 6 | 2 | | Format version

#### Format versions

| Version | Description
| --- | ---
| "00" | Supported as of Tiger
| "01" | Supported as of Leopard
| "0x" | Supported as of Snow Leopard, where x is any character

### Object table

The object table consists of:

* zero or more objects

Objects are of variable size and consist of:

* an object maker byte
* (optional) object data

#### Object marker byte

| Value | Identifier | Description
| --- | --- | ---
| 0x00 | kCFBinaryPlistMarkerNull | Empty value (NULL)
| | |
| 0x08 | kCFBinaryPlistMarkerFalse | Boolean False
| 0x09 | kCFBinaryPlistMarkerTrue | Boolean True
| | |
| 0x0f | kCFBinaryPlistMarkerFill | Unknown (Fill byte?)
| 0x1# | kCFBinaryPlistMarkerInt | Integer, where 2^# is the number of bytes
| 0x2# | kCFBinaryPlistMarkerInt | Floating point, where 2^# is the number of bytes
| | |
| 0x33 | kCFBinaryPlistMarkerDate | Date and time value, which is stored as a 64-bits floating point that contains the number of seconds since Jan 1, 2001 00:00:00 UTC
| | |
| 0x4# | kCFBinaryPlistMarkerData | Binary data, where # is the number of bytes. If # is 15 then the object marker byte is followed by a 32-bit integer that contains the size of the data.
| 0x5# | kCFBinaryPlistMarkerASCIIString | ASCII string, where # is the number of characters. If # is 15 then the object marker byte is followed by an integer object that contains the number of characters in the string. The string is stored in ASCII (with codepage?) without an end-of-string marker
| 0x6# | kCFBinaryPlistMarkerUnicode16String | Unicode string, where # is the number of characters. If # is 15 then the object marker byte is followed by an integer object that contains the number of characters in the string. The string is stored in UTF-16 big-endian without an end-of-string marker
| 0x7# | | Unused
| 0x8# | kCFBinaryPlistMarkerUID | UID, where # + 1 is the number of bytes.
| 0x9# | | Unused
| 0xa# | kCFBinaryPlistMarkerArray | Array of objects, where # is the number of elements. If # is 15 then the object marker byte is followed by an integer object that contains the number of elements in the array.
| 0xb# | | Unused
| 0xc# | kCFBinaryPlistMarkerSet | Set of objects, where # is the number of elements. If # is 15 then the object marker byte is followed by an integer object that contains the number of ele,emts in the set.
| 0xd# | kCFBinaryPlistMarkerDict | Dictionary of key value pairs, where # is the number of key value pairs. If # is 15 then the object marker byte is followed by an integer object that contains the number of key value pairs in the dictionary.
| 0xe# | | Unused
| 0xf# | | Unused

#### Array object

The array object consists of:

* array object marker with number of elements
* array of object references that identify the element objects.
* the element object data

The byte size of the object reference is defined in the trailer. An object
reference of 1 will refer to the first object in the (object) offset table.

#### Set object

The set object consists of:

* set object marker with number of elements
* array of object references that identify the element objects.
* the element object data

The byte size of the object reference is defined in the trailer. An object
reference of 1 will refer to the first object in the (object) offset table.

#### Dictionary object

The dictionary object consists of:

* dictionary object marker with number of key and value pairs
* array of key references that identify key objects.
* array of object references that identify the value objects.
* the key/value object data

The byte size of the key and object reference is defined in the trailer. A key
and object reference of 1 will refer to the first object in the (object) offset
table.

### (Object) offset table

The offset table consists of an array of offsets. The trailer defines:

* The location of the offset table
* The offset byte size
* The number of offsets in the table

The offset values are relative from the start of the file.

### Binary plist trailer

The binary plist trailer (CFBinaryPlistTrailer) is 32 bytes in size and
consists of:

| Offset | Size | Value | Description
| --- | --- | --- | ---
| 0 | 5 x 1 | 0 | Unknown (0-byte values)
| 5 | 1 | 0 | Unknown (Sort version)
| 6 | 1 | | Offset byte size
| 7 | 1 | | Key and object reference byte size
| 8 | 8 | | Number of objects
| 16 | 8 | | Root (or top-level) object
| 24 | 8 | | Offset table offset, where the offset is relative to the start of the file

## XML plist format

A XML plist file consists of:

* optional XML declaration
* optional Document Type Definition (DTD)
* plist root XML element
* key-value pair XML elements

For example:

```
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist SYSTEM "file://localhost/System/Library/DTDs/PropertyList.dtd">
<plist version="1.0">
...
</plist>
```
