# Object index

The object index file (`.aidx`) contains information about the placement of objects within the package root.

This file can be used / handled in two different ways:

- **Standalone**: Directly handling a `.aidx` file
- **Via the object database**: Using OID references that contain the index file contents

It itself is checked into the object database and can be referenced by the `package.toml` file.

On a high level, this file represents a sequence of instructions on how to walk a filesystem tree and what to do at the various points in time. This makes storage of this index quite efficient due to there being no need to store the full paths to all files. On the opposite side is the limitation that this file must be read sequentially and almost certainly requires tooling to manipulate. This is the reasoning behind this file being in binary format.

# Structure

The file is stored in `little-endian` binary format, using `UTF-8` for all of its strings, which are not `NULL` - terminated as their length is always defined.

The file starts with the following structure:

| Offset | Count | Description        |
| :----: | :---: | ------------------ |
|   0    |   4   | File magic: `AIDX` |
|   4    |   1   | Version: `0x00`    |

After this header, the file starts working in a instruction form. The current virtual working directory (`VWD`) gets retained between commands to allow navigation of the index like a filesystem in a shell.

| Offset | Count | Description |
| :----: | :---: | ----------- |
|   0    |   1   | Command     |

The command determines the next bytes:

- `0x00` - [`DirectoryUP` - Go up one directory](#0x00---directoryup)
- `0x10` - [`Directory` - Change into a directory, creating it if it doesn't exist](#0x10---directory)
- `0x20` - [`File` - Create a file using an object](#0x20---file)
- `0x30` - [`Symlink` - Create a symlink](#0x30---symlink)

## 0x00 - DirectoryUP

> This command has no additional data

Moves `VWD` up one directory to the parent of the current contents of `VWD`.

## 0x10 - Directory

Additional data structure:

| Offset | Count | Description             |
| :----: | :---: | ----------------------- |
|   0    |   4   | UNIX user id - `uid`    |
|   4    |   4   | UNIX group id - `gid`   |
|   8    |   4   | UNIX file mode - `mode` |
|   12   |   4   | Name length             |
|   16   |       | Name                    |

Pushes `Name` onto `VWD` and tries to change into that directory, creating it using the UNIX information in the struct if it doesn't exist.

## 0x20 - File

Additional data structure:

| Offset | Count | Description             |
| :----: | :---: | ----------------------- |
|   0    |   4   | UNIX user id - `uid`    |
|   4    |   4   | UNIX group id - `gid`   |
|   8    |   4   | UNIX file mode - `mode` |
|   12   |   4   | Name length             |
|   16   |   4   | OID length              |
|        |       | Name                    |
|        |       | Object id `OID`         |

Creates a file called `Name` by pushing `Name` onto `VWD` and using that as the path to place the file at. The newly created file uses information from the `UNIX*` fields in this struct and fills its contents with the contents provided by the object `OID`.

> **Note**
>
> The object id `OID` is represented as a byte string as returned by the hashing algorithm. It is not represented in string form!

## 0x30 - Symlink

Additional data structure:

| Offset | Count | Description             |
| :----: | :---: | ----------------------- |
|   0    |   4   | UNIX user id - `uid`    |
|   4    |   4   | UNIX group id - `gid`   |
|   8    |   4   | UNIX file mode - `mode` |
|   12   |   4   | Name length             |
|   16   |   4   | Target length           |
|        |       | Name                    |
|        |       | Target                  |

Creates a symlink named `Name` pointing to `Target` by pushing `Name` onto `VWD` and using that as the path to place the symlink at. The newly created symlink uses information from the `UNIX*` fields in this struct.
