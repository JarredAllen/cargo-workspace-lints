Enforce Package use Workspace Lints
---

Rust dropped a cool new feature in `1.74.0` that enables specifying levels for `rustc` and `clippy`
lints in your workspace `Cargo.toml` file, and then the packages in your workspace can all inherit
lints from the same source. However, each package in the workspace needs to include a
`lints.workspace = true` line, or else it won't take those lints.

Enter `cargo-workspace-lints`! Once you install it, all you need to do is run `cargo
workspace-lints` in your workspace, and it will check all the packages in your workspace.

For an example, you can use it on this crate! This crate is not a workspace, so it produces a nice
error message:
```
$ cargo install cargo-workspace-lints --locked
...
$ git checkout https://github.com/JarredAllen/cargo-workspace-lints.git
$ cd cargo-workspace-lints
$ cargo workspace-lints
Failed to validate:
Failing packages:
* Package cargo-workspace-lints 0.1.0 (path+file://home/user/cargo-workspace-lints):
     No `workspace.lints` field found
$ echo $?
1
```

Run `cargo workspace-lints --help` for full details of the options with the command.
