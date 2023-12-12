# Branch

The `branch` tool is the builder. It is used to transform blueprints of packages, so called `formulae` into installable packages by gathering the neccessary files files and dependencies into a build environment to build the binaries that can then be installed by users.

> [!NOTE]
> 
> `branch` mounts filesystems and chroots, so it will probably require `root` permissions to perform its actions.

> [!CAUTION]
> Please be careful when executing commands with `root` permissions and double-check the arguments you pass to this program!

# Usage

The `branch` command needs 2 arguments to work properly:

- `toolchain`: A directory that can be appended with `/bin` to set the `PATH` to. The binaries in this path will be available to the build scripts by default.

- `formula`: A `.toml` file that is parseable as a formula. This is the description for the package that will be built by the builder.

# Dependencies

`branch` runs a finished package through a set of validators that produce suggested actions in form of executable commands. The following programs get used / assumed:

- `patchelf`: The `ELF` patching utility from the `NixOS` project

- `sed`: The stream editor is used to execute regexes on regular files

- `strip`: The command from the `binutils` package. Used to strip binary files to reduce their size

> [!NOTE]
> 
> The program assumes that these dependencies exist and will output errors if they are not available!

# Build pipeline

`branch` follows a strict pipeline of actions to build a package. It is described [here](pipeline.md).
