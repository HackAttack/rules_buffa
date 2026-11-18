# rules_buffa

Bazel rules for building protos in Rust with [buffa](https://github.com/anthropics/buffa).

`rust_buffa_library` compiles each `proto_library` in a target’s dependency graph into its own Rust crate, the way [`rules_rust_prost`](https://github.com/bazelbuild/rules_rust/tree/main/extensions/prost)’s `rust_prost_library` does for prost. Service stubs come from [ConnectRPC](https://github.com/connectrpc/connect-rust) through an opt-in toolchain.

```python
bazel_dep(name = "rules_buffa", version = "{SEE_RELEASE_NOTES}")
```

```python
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_buffa//:defs.bzl", "rust_buffa_library")

proto_library(
    name = "my_proto",
    srcs = ["my.proto"],
)

rust_buffa_library(
    name = "my_rs_buffa",
    proto = ":my_proto",
)
```

Toolchain options, per-target transforms, services, and the differences from `rules_rust_prost` are documented in [`defs.bzl`](defs.bzl).

The rules load private `rules_rust` APIs to compile generated crates, so each release supports the `rules_rust` version in [`MODULE.bazel`](MODULE.bazel).
