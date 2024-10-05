# Object files

An AcaciaLinux object is a binary file format that stores any kind of data that can be labelled and compressed to be inserted and handled within the object database.

AcaciaLinux speaks exclusively in objects, meaning that everything that is stored by the AcaciaLinux tools gets packed into objects and is transferred and stored in this way only.

## Binary structure

An object is stored in binary format, as that saves on storage and makes parsing faster and less error-prone.
All data types are stored in `little-endian` format and strings are **always** valid `UTF-8` without a null terminator as the length is always known.

The following is the initial layout of an object in all of its versions that is guaranteed:

| Offset | Count | Description        |
| :----: | :---: | ------------------ |
|   0    |   4   | File magic: `AOBJ` |
|   4    |   1   | Version: `0x00`    |

## Version 0

Version 0 (`0x00`) continues with the following layout:

| Offset | Count | Description              |
| :----: | :---: | ------------------------ |
|   0    |  32   | Object ID                |
|   32   |   2   | Object type              |
|   34   |   2   | Compression type         |
|   36   |   4   | Dependencies count (`d`) |
|   40   |   8   | Data length (`b`)        |
|   48   |  `d`  | Dependencies count       |
| 48+`?` |  `b`  | Data                     |

### Object Type

The object type gives the consumer of the object some hints about the contents of the object and how to further process it.
This allows for grouping of files that are treated equally and to adust the processing of the file depending on the type of file at hand.

The type is divided into a `class` and a `type` to separate the namespace.

#### `0x00`: Miscellaneous

- `0x00`:`0x00` => Unknown object

#### `0x01`: AcaciaLinux specific

Objects within this namespace are specific to the AcaciaLinux system and reserved:

- `0x01`:`0x00`: Unknown AcaciaLinux specific object
- `0x01`:`0x10`: Package list
- `0x01`:`0x20`: Formula
- `0x01`:`0x30`: Package
- `0x01`:`0x40`: Index

### Compression type

Object data can be compressed before being stored in the object container.
This field gives information about the compression used when creating the object.

Do note that the compression does not change the object id as it is calculated from the raw and uncompressed binary data.

- `0x00`: No compression
- `0x01`: Xz compression

### Dependencies

The dependencies field lists the objects that this object needs to work properly.
It does this by concatenating the following things into a long list:

| Offset | Count | Description       |
| :----: | :---: | ----------------- |
|   0    |  32   | Object ID         |
|   32   |   2   | Path length (`p`) |
|   34   |  `p`  | Path              |

After this structure, the next dependency starts until the dependencies count is reached.
