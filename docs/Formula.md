# The formula

A formula in the AcaciaLinux ecosystem represents a file and possibly some accompanying files that describe how to build, test and archive a package.

## Formula files

A formula file is written in [toml](https://toml.io) and ingested into the object database using the `branch ingest` command.

The following is a schema of a one-package formula:
```toml
file_version = 1

name = "<Name of the package / project>"
version = "<Version of the packaged project>"
description = "<Formula description>"

host_dependencies = []
target_dependencies = []
extra_dependencies = []

[[sources]]
url = "<Source URL if some>"
dest = "<Destination for the source file relative to formula file>"
extract = true # Whether or not to extract the source if possible

prepare = "<commands to prepare the formula>"
build = "<commands to build the formula's binaries>"
check = "<commands to check the formula's built binaries>"
package = "<commands to prepare the formula's binaries for shipping>"
```

### Dependencies

A formula can specify 3 types of dependencies:

**Host dependencies**

The `host_dependencies` are used and run on the building computer.
They are installed in the builder's (host) architecture and typically include build tools and compilers.

**Target dependencies**

The `target_dependencies` are intended for the running computer - the target of the compiler binaries.
They typically include libraries and programs that the formula's binaries rely on.
The builder checks the dependencies on the file-level and includes them in the package's dependency list if they are needed by some file.

**Extra dependencies**

The `extra_dependencies` come into play when the builder can't infer the dependencies by looking at the files.
Sometimes, the dependencies are hidden or the builder cannot (yet) infer them automatically.
This is where the extra dependencies come into play to force the package to include "blind" dependencies that have no reason to exist according to the builder, but the maintainer tells the system that they are needed anyway.

### Package Steps

A formula can define 4 package steps:

- `prepare`: Prepare the source for compilation
- `build`: Build the binary artifacts
- `check`: Check that the produced binaries work as intended
- `package`: Copy the binaries and artifacts to a package directory to be shipped via the package distribution system.

These 4 steps build upon each other using mechanisms from the `overlayfs` in that a build step receives the `upper` directory of the previous build step as a `lower` directory.
In practice, this means that the `build` method has the `upperdir` of the `prepare` method in its `lowerdir` list, as it builds upon and needs the prepared sources from the previous build step.

## Multi-package formula

AcaciaLinux supports the concept of multi-package-formulae.
They are formulae that emit multiple packages.
By default, the packaging process creates an invisible package named after the formula that inherits all attributes from the parent formula.

Multi-package-formulae allow for more fine-grained control over the build process of multiple packages that originate from the same source.
This can be helpful if a project builds multiple binaries and libraries that make sense to be split out into multiple packages.

The single-package formula is secretly a multi-package formula in that `branch` takes the formula's name and layout and creates an internal package that matches these attributes.
As soon as the `packages` map is no longer empty, `branch` stops this behavior and the user has full control over the provided packages.

For a multi-package formula, the schema looks as follows:

```toml
file_version = 1

name = "<Name of the package / project>"
version = "<Version of the packaged project>"
description = "<Formula description>"

arch = ["<List of supported architectures>"]

host_dependencies = []
target_dependencies = []
extra_dependencies = []

[[sources]]
url = "<Source URL if some>"
dest = "<Destination for the source file relative to formula file>"
extract = true # Whether or not to extract the source if possible

prepare = "<commands to prepare the formula>"
build = "<commands to build the formula's binaries>"
check = "<commands to check the formula's built binaries>"
package = "<commands to prepare the formula's binaries for shipping>"

[packages."<package name>"]
name = "<Package name>"
version = "<Package version>"
description = "<Package description>"

extra_dependencies = []

prepare = "<commands to prepare the package>"
build = "<commands to build the package's binaries>"
check = "<commands to check the package's built binaries>"
package = "<commands to prepare the package's binaries for shipping>"
```

Each package inherits all unspecified properties from the parent formula.

A package has only `extra_dependencies`, as all the other dependencies are specified by the formula.

The build steps are handled in a special way in that each and every step gets executed using its own `overlayfs`. If the formula specifies a `prepare` method and the package does so, too, the formula's method gets executed and that upper directory included as a lower directory in the package's method.
The same happens for all the other methods, building upon each other.
