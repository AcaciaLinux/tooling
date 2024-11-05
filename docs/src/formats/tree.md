# Tree

AcaciaLinux stores file system hierarchies in a tree-like form that is similar to that used by git.
Each tree is an index of a directory and its contents that does not descend recursively.
Nested subdirectories are implemented in the form of subtrees.
This approach increases the chance of same directory structures that can be addressed via the same object id.

# Structure

The file is stored in `little-endian` binary format, using `UTF-8` for all of its strings, which are not `NULL` - terminated as their length is always defined.

The file starts with the following structure:

| Offset | Count | Description        |
| :----: | :---: | ------------------ |
|   0    |   4   | File magic: `ALTR` |
|   4    |   1   | Version: `0x00`    |

After this header, the file starts working in a instruction form. The current virtual working directory (`VWD`) gets retained between commands to allow navigation of the index like a filesystem in a shell.

| Offset | Count | Description |
| :----: | :---: | ----------- |
|   0    |   1   | Command     |

The command determines the next bytes:

- `0x01` - [`File` - Create a file using an object](#0x01---file)
- `0x02` - [`Symlink` - Create a symlink](#0x02---symlink)
- `0x05` - [`Subtree` - Place a subtree](#0x05---subtree)

## 0x01 - File

Additional data structure:

| Offset | Count | Description             |
| :----: | :---: | ----------------------- |
|   0    |  32   | Object id `OID`         |
|   32   |   4   | UNIX user id - `uid`    |
|   36   |   4   | UNIX group id - `gid`   |
|   40   |   4   | UNIX file mode - `mode` |
|   44   |   4   | Name length             |
|        |       | Name                    |

Creates a file called `Name` by pushing `Name` onto `VWD` and using that as the path to place the file at. The newly created file uses information from the `UNIX*` fields in this struct and fills its contents with the contents provided by the object `OID`.

> **Note**
>
> The object id `OID` is represented as a byte string as returned by the hashing algorithm. It is not represented in string form!

## 0x02 - Symlink

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

## 0x05 - Subtree

Additional data structure:

| Offset | Count | Description             |
| :----: | :---: | ----------------------- |
|   0    |  32   | Object id `OID`         |
|   32   |   4   | UNIX user id - `uid`    |
|   36   |   4   | UNIX group id - `gid`   |
|   40   |   4   | UNIX file mode - `mode` |
|   44   |   4   | Name length             |
|   48   |       | Name                    |

This places the contents of a tree with `OID` at `Name`, effectively creating a subdirectory.

> **Note**
>
> The object id `OID` is represented as a byte string as returned by the hashing algorithm. It is not represented in string form!
