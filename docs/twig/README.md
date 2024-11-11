# Twig

The `twig` tool is a low-level plumbing utility.
It provides access to various utilities that do not fit in the other tools.
It can be seen as a small brother to `branch` (hence the name).

Twig works using subcommands, of which the following are available:

- [`odb`](#object-database-access-twig-odb): Interact with the object database in a low-level way

- [`tree`](#index-utilities-twig-tree): Work with trees

> [!TIP]
> Twig assumes the acacia directory to exist at the current user's home (`~/.acacia`).
> This behavior can be changed by using the `--home <ACACIA_HOME>` option to steer `twig` to another acacia directory.

> [!TIP]
> Normally, `twig` will not print much information about the inner workings, this can be changed by the `-v {0;1;2;3}` flag, where increasing numbers increase the verbosity of the program.

## Object database access (`twig odb`)

The `twig odb` command has the following subcommands:

- [`twig odb get`](#retrieving-objects-from-the-object-database): Get the contents of an object from the object database

- [`twig odb put`](#inserting-objects-into-the-object-database): Put a new object into the object database

### Retrieving objects from the object database

This subcommand facilitates retrieving object contents from the object database.

```
twig odb get [--output <FILE>] <OID>
```

> [!TIP]
> By default, this command outputs the contents of the objects to `STDOUT`.
> This behavior can be changed by using the `--output <FILE>` option.
> This puts the contents of the object into `<FILE>`.

### Inserting objects into the object database

This subcommand facilitates inserting new objects into the object database.

```
twig odb put [--compression {none;xz}] [--force] <PATH>
```

> [!TIP]
> Normally, twig checks for an already existing object in the database.
> The `--force` flag will force twig to overwrite the existing object.

## Tree utilities (`twig tree`)
