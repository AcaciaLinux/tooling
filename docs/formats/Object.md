# The Acacia Object Format

The AcaciaLinux object is a binary format that is used to store and exchange Acacia Objects that build the backbone of the AcaciaLinux system.
Objects can hereby contain arbitrary data.
The object is only concerned with how and how efficient it can store and exchange this data.
To reduce storage and transmission bandwidth demands, the object data can be compressed to facilitate faster transfers and less storage overhead.

# Binary Structure

The structure of this format starts with a small header that is fixed and will never change.
Starting from it, the parser can then infer the following data layout from the version described in the `Version` field.

| Octets |                 Description                 |
| :----: | :-----------------------------------------: |
|   4    | File magic: `AOBJ` [0x41, 0x4F, 0x42, 0x4A] |
|   1    |                   Version                   |

The following versions are defined:

- **0** - Reserved and unused
- **1** - [Version 1](#version-1)

# Version 1

Version 1 of the object format defines the following additional structure:

This version of the object format uses **little-endian** encoding for multi-byte values.

| Octets |                   Description                    |
| :----: | :----------------------------------------------: |
|   1    | [Compression Type](#version-1-compression-types) |
|   8    |              Compressed Length (d)               |
|   8    |                 Extracted Length                 |
|   d    |                       Data                       |

## Version 1 Compression Types

The following compression types are defined:

- **0x00** - No compression, raw data
- **0x01** - LZMA
