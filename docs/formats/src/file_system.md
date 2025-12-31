# File system formats

A file system format is used to manage the storage of files.

## Terminology

* **File entry (file system entry)**: an object that represent an element within
  the file system, such as a file or directory. A file system typically stores
  metadata of a file entry, such as the name, size, permissions, date and time
  values, and location of the content.
* **Data fork (or data stream)**: a file system object that represents the
  content of a file entry. NTFS and HFS support multiple data forks (or data
  streams) for an individual file entry.
* **Extended attribute**: A file system object that represents additional (or
  extended) metadata of an individual file entry.
* **Reparse point**: a file system object that redirects to another location or
  implementation (filter driver), such as Windows Overlay Filter (WOF)
  compression. NTFS and ReFS support reparse points.

## Formats

* [Apple File System (APFS)](apfs.md)
* [Extended File System (ext)](ext.md)
* [Extensible File Allocation Table (exFAT)](exfat.md)
* [File Allocation Table (FAT)](fat.md)
* [Hierarchical File System (HFS)](hfs.md)
* [Macintosh File System (MFS)](mfs.md)
* [New Technologies File System (NTFS)](ntfs.md)
