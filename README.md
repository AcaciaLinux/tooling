# AcaciaLinux tooling

This repository contains tooling for the AcaciaLinux distribution, implementing the new concept, available [here](https://github.com/AcaciaLinux/docs).

If you want to get involved, read the [CONTRIBUTING](CONTRIBUTING.md) article!

# Tools

## branch

The `branch` tool is the builder that is used to build packages.

Further information and documentation on branch can be found [here](docs/branch/README.md).

**Invocation**

```bash
branch --toolchain <toolchain> <formula.toml>
```

## twig

The `twig` tool is a low-level utility program to aid with internal things.
It provides common utility functionality that can be interesting for scripting.

Further information an documentation on `twig` can be found [here](docs/twig/README.md).
