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

# Output

Branch will output a list of commands on its `stdout` stream. They can be executed immediately to patch the package file to make them compatible with the AcaciaLinux distribution.

The most common usage for `branch` is to pipe its `stdout` output to a file or an interpreter to apply the suggested patches:

```bash
(sudo) branch --toolchain [toolchain] [formula.toml] | (sudo) sh -s # This will immediately apply all patches
```

> [!NOTE]
> 
> `branch` will redirect all `stdout` streams to `stderr` to free up the main `stdout` stream for redirecting.

# Dependencies

`branch` runs a finished package through a set of validators that produce suggested actions which can be transformed and output as shell commands. The following programs get used / assumed:

- `patchelf`: The `ELF` patching utility from the `NixOS` project

- `sed`: The stream editor is used to execute regexes on regular files

The program assumes that these dependencies exist, but does not require them to be available to it. It generates an output regardless of the availability of these binaries. The user can then whether to apply the actions or to modify them according to its needs.

# Build pipeline

`branch` follows a strict pipeline of actions to build a package. It is described [here](pipeline.md).
