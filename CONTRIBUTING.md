# Contributing to this repository

Contributions are welcome, we certainly appreciate everyone that is part of the project and contributes to making it better.

Before diving into the code, make sure there isn't already someone doing the same thing, do not waste your time! Useful tools for this are the issues page and looking into branches.

If you want to fix an issue, make sure it is actually fixable and not marked as will not fix. Contributions that fall into this category likely won't be merged!

Once you are done, be sure to write a good explanation in your pull request and cross your fingers.

# Code

There are some coding guidelines we like to be followed

### Rust

Use `rustfmt`. The formatter ensures consistent and readable code. Use `cargo fmt` before committing!

To ensure the compiler is happy and the written Rust code is sane, use the `cargo clippy` command to see if anything could be improved.

Make the compiler happy!

### Documentation

Document as much as possible! Documentation helps the maintainers, as well as users and other contributors to understand what your (or others) code does and should do. Rust's documentation is a really nice tool, so use `cargo doc` to see if your modules, traits, structs and functions are well documented for a user to understand.

# GIT

### Committing

Commits should be clear and concise, explaining what the author did to the repository.

If a committ changes something in a specific module, prefix the message with its path (`<module>::<module>...`). This allows the reader to infer the changed directories at a glance.

### Merging

When merging commits, do make sure that the merge is a commit, so the tree reflects what has been merged when.

### Rebasing

When rebasing, make sure to use the `--committer-date-is-author-date` flag to keep the dates of all the commits. This helps readers and maintainers to track down changes in time.


