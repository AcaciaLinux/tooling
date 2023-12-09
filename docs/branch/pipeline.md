# Branch build pipeline

The `branch` build tool follows the pipeline as described in this document:

1. Determine build architecture

2. Parse local installed packages and index them

3. Parse the formula

4. Create a build environment

5. Build the package
   
   1. Collect sources by downloading and extracting them
   
   2. Run build steps: `prepare`, `build`, `check`, `package`
   
   3. Validate the package and populate dependencies
   
   4. Emit action commands
   
   5. Generate `package.toml` metadata
   
   6. Populate `link/` directory

6. Tear down the build environment

## 1. Determine build architecture

The first step is to determine the architecture the package is built for. The `formula` lists the compatible architecures in the `package.arch` field. To determine if a package can be built, the builder has to know the target architecture.

The architecture is inferred by using the [`uname`](https://crates.io/crates/uname) crate and its `machine` field.

> [!TIP]
> 
> The build architecture can be overridden by adding the `--arch <architecture>` option to the `branch` command line. This allows for cross compilation of packages.

## 2. Parse local installed packages and index them

For `branch` to be able to compose a build environment with all the dependencies that are required, it has to know what dependencies are installed and in what version.

To accomplish this, `branch` will use the package index at `<DIST_DIR>/packages.toml`.

> [!TIP]
> 
> The path to the package index can be overridden by adding the `--package-index <path>` option to the `branch` command line.

By looking into this file, `branch` gets information about the name, version and architecture of the installed packages. It will use these informations to locate them using the `<DIST_DIR>/` by appending the architecture, name and version of the packages.

> [!TIP]
> 
> The `DIST_DIR` can be overridden by adding the `--dist-dir <path>` option to the `branch` command line.

Once a package has been located, `branch` will move on to parse its `package.toml` metadata file to obtain more detailed information about the package.

After parsing package metadata, `branch` will index the package and create a detailed tree of all the files in the package, their type and contents. This allows `branch` to work out which package provides which file and thus infer dependencies at the validation stage.

## 3. Parse the formula

This step is quite simple: `branch` will use the formula argument to get to the formula file and parse it. It will also note down the parent directory of the formula file, because it will later be mapped into the build root so the formula build steps can access all the files in the directory of the formula.

## 4. Create a build environment

To construct a build environment, `branch` will create the `overlay/<build id>` directory in its working directory.

> [!TIP]
> 
> The working directory can be overridden by adding the `--workdir <path>` option to the `branch` command line

The `branch` program will then move on to creating an `overlayfs` mount consisting of the `root/` directories of the `target_dependencies` and any additional lower directories added using the `--overlay-dirs <path>` option in the `branch` command line.

A second `overlayfs` mount will be created to pass the formula parent directory to the build root to make the formula available.

A `rw` `bind` mount will be created to map the package archive directory into the build root to get the packaged files out of the build root. The build root's internal path (joied with `data/`) will be provided under the `$PKG_INSTALL_DIR` environment variable.

A `ro` `bind` mount will be created to pass the `DIST_DIR` into the build root. This makes the host's toolchain available to the build process. The `chroot`'s `PATH` variable will be set to `<TOOLCHAIN>/bin` to expose the toolchain binaries to the build process.

The following virtual kernel filesystems (`vkfs`) will be mounted into the build root to make it able to be `chroot`'ed into:

- `/dev` (bind)

- `/dev/pts` (bind)

- `/sys` (sysfs)

- `/proc` (proc)

- `/run` (tmpfs)

## 5. Build the package

This is the point where the user's code will start running to build a package

## 5.1. Collect sources by downloading and extracting them

> [!NOTE]
> 
> All strings in the `package.sources` section get the following variables replaced:
> 
> - `$PKG_NAME`: The package name
> 
> - `$PKG_VERSION`: The package version
> 
> - `$PKG_ARCH`: The build architecture

The `package.sources` field in the formula contains a `url` field, which will be used to fetch sources by using the `libcurl` library.

The package maintainer can provide a custom destination path by using the `dest` field. This will change the filename of the resulting file.

> [!WARNING]
> 
> The `dest` field **HAS** to be relative! It will be joined to the working directory.

If desired, `branch` can extract archives automatically by setting the `extract` field to `true` (default). Do note that if the file is not extractable, `branch` will error out and abort the build process.

## 5.2. Run build steps: `prepare`, `build`, `check`, `package`

Now, `branch` will run the 4 build steps as described in the formula and packaging specification of the AcaciaLinux project. Please refer to them for further information.

The `chroot` environment executes `env` and `sh -e` to execute the commands. The `-e` flag will force the interpreter to cancel a script if any subcommand fails.

If any of the build steps exits with a non-0 exit code, `branch` will abort the operation.

## 5.3. Validate the package and populate dependencies

After the package has been built, `branch` will index the package contents and run them through a set of validators, as desribed in the AcaciaLinux documentation. Please refer to it for further information on these steps.

## 5.4. Emit action commands

After validation, `branch` will transform the actions, as suggested by the validation phase to a set of runnable commands and outputs them to `stdout` for them to be piped to a file or immediately into an interpreter.

## 5.5. Generate `package.toml` metadata

Each package has its metadata file (`package.toml`) that holds informations about the dependencies the package has been built against, its maintainer, build-id and other useful information for the package manager and user.

This is the step where `branch` will use all of the informations in the formula, the discovered dependencies in the validation phase and additional user inputs to generate this file.

## 5.6. Populate `link/` directory

A package has a `link/` directory where it will look for dependencies. The patches suggested by the validation point to this directory instead of the real path to the dependency. This allows the package manager or the user to adjust dependencies if so-desired.

This step is not strictly neccessary but it helps to make manual installation of packages easier. The `link/` directory will be populated with symlinks to the default path of the package dependencies as constructed by using the dist directory, the package architecture, name and version.

> [!NOTE]
> 
> The dist directory that is used to construct the default dependency location cannot be altered by the user! It is set in stone at compile time!

## 6. Tear down the build environment

After all build operations have succeeded, `branch` will tear down the build environment by unmounting all mounts pointing to the build root to free up resources. This step is neccessary to make the build root directory and the overlay directory accessible to users again. If there are still mounts going into them, they cannot be removed to free up space or archived.


