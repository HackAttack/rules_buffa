"""# rules_buffa

These build rules are used for building [protobufs][protobuf] in [Rust][rust] with Bazel
using [buffa][buffa]

[rust]: http://www.rust-lang.org/
[protobuf]: https://developers.google.com/protocol-buffers/
[buffa]: https://github.com/anthropics/buffa

## Rules

- [rust_buffa_library](#rust_buffa_library)
- [rust_buffa_toolchain](#rust_buffa_toolchain)
- [rust_buffa_transform](#rust_buffa_transform)

## Setup

```python
bazel_dep(name = "rules_buffa", version = "{SEE_RELEASE_NOTES}")
```

Each `proto_library` is compiled into one Rust crate which can be consumed like
any other Rust dependency.

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

The `buffa_srcs` output group on `rust_buffa_library` exposes the generated
crate roots, so other tooling, e.g. a Cargo `build.rs`, can consume the exact
source Bazel compiles.

For additional information about Bazel toolchains, see [here](https://docs.bazel.build/versions/master/toolchains.html).

#### <a name="custom-buffa-deps">Customizing `buffa` Dependencies

The default toolchain uses the [`buffa`], [`buffa-types`], [`buffa-descriptor`], and
[`buffa-codegen`] crates vendored by this module. To use different versions, build the
codegen binary from the source exported by this module and declare a
`rust_buffa_toolchain` with your own crates (e.g. from a
[crate_universe](https://bazelbuild.github.io/rules_rust/crate_universe.html) repository).

[`buffa`]: https://crates.io/crates/buffa
[`buffa-types`]: https://crates.io/crates/buffa-types
[`buffa-descriptor`]: https://crates.io/crates/buffa-descriptor
[`buffa-codegen`]: https://crates.io/crates/buffa-codegen

```python
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@rules_buffa//:defs.bzl", "rust_buffa_toolchain")

rust_binary(
    name = "codegen",
    srcs = ["@rules_buffa//private:codegen.rs"],
    edition = "2021",
    deps = [
        "@crates//:buffa",
        "@crates//:buffa-codegen",
    ],
)

rust_buffa_toolchain(
    name = "buffa_toolchain_impl",
    codegen = ":codegen",
    opts = ["json=true"],
    runtime = [
        "@crates//:buffa",
        "@crates//:buffa-types",
        "@crates//:buffa-descriptor",
        "@crates//:serde",
    ],
)

toolchain(
    name = "buffa_toolchain",
    toolchain = ":buffa_toolchain_impl",
    toolchain_type = "@rules_buffa//:toolchain_type",
)
```

`opts` passes `key=value` options to buffa's codegen. Supported keys:

- Booleans: `views`, `lazy_views`, `unknown_fields`, `json`, `text`,
  `arbitrary`, `allow_message_set`, `strict_utf8_mapping`, `register_types`,
  `with_setters`, `idiomatic_imports`, `idiomatic_field_names`,
  `idiomatic_enum_aliases`, `gate_impls`, and `shared_descriptor_pool` (the
  last requires `reflection` and embeds one descriptor pool at the crate
  root shared by all packages).
- `unbox_oneof` and `unbox_oneof_in=<proto path>`: store message oneof
  variants inline instead of boxing them.
- `reflection` (bool) and `reflect_mode=off|bridge|vtable`: reflection impls.
- `type_name_prefix=<Prefix>`: prepend a prefix to generated type names.
- `open_enums_in=<proto path>` and `override_feature_in=<proto
  path>=<feature>:<value>`: editions feature overrides (`enum_type:open` is
  the only supported override).
- `extern_path=<proto path>=<rust path>`: map a proto package or type to an
  external Rust path. A user entry overrides the automatic manifest mapping
  for that exact type; set it on the toolchain when several crates must
  agree.
- Field representations `bytes_type`, `string_type`, `map_type`, `box_type`,
  and `repeated_type`, each accepting a built-in name (`vec`, `bytes`,
  `string`, `hash_map`, `btree_map`, `box`, or `inline`, as applicable), plus
  `<kind>_custom=<rust path>` for a custom type and
  `<kind>_in=<proto path>=<repr>` / `<kind>_custom_in=<proto path>=<rust
  path>` for path-scoped rules.
- Attributes `type_attribute`, `field_attribute`, `message_attribute`,
  `enum_attribute`, and `oneof_attribute`, each `<proto path>=<attribute>`.
- `json_feature`, `views_feature`, `text_feature`, and `reflect_feature`:
  rename the crate feature a `gate_impls`-gated impl kind is conditioned on.

Generated crate roots `pub use` each dep crate they compile against, so
dependents can also reach them as `<crate>::<dep_crate>::...`. Setting
`include_transitive_deps` on `rust_buffa_toolchain` additionally adds all
transitive dep crates to the crate group, so dependents can name them
directly.

`crate_features` on `rust_buffa_toolchain` (and `rust_buffa_transform`)
passes `--cfg feature="<name>"` to rustc; it is needed by options that gate
generated code on crate features: `arbitrary=true` is always gated, and
`gate_impls=true` gates the `json`, `views`, `text`, and `reflect` impls.

`exclude_package` and `file_per_package` do not apply: each `proto_library`
generates only its own files, into one crate root.

Options that need runtime support require matching features on the `runtime`
crates:

- `json`: `json` on `buffa`, `buffa-types`, and `buffa-descriptor`, plus
  `serde` in `runtime`.
- `text`: `text` on `buffa` and `buffa-descriptor`.
- `arbitrary`: `arbitrary` on all three, the `arbitrary` crate in `runtime`,
  and `crate_features = ["arbitrary"]`.
- `reflection`: `reflect` on `buffa-descriptor` and `buffa-types`.
- Views (on by default): `views` on `buffa-descriptor`.

The `buffa-descriptor` features matter only when protos use
`descriptor.proto` message types as fields. The module's vendored crates
enable only what its own toolchains use (JSON, descriptor views, and
reflection), so other options, and extras such as the `jiff`/`chrono`
conversions in `buffa-types`, need your own crates in a custom toolchain.
Two limits come from buffa itself: `Api`/`Method`/`Mixin`/`Type`/
`SourceContext` have no serde impls, so `json=true` cannot compile a message
embedding them, and `descriptor.proto`/`plugin.proto` types have no reflect
impls, so `reflection=true` cannot either.

Lastly, you must register the toolchain in your `MODULE.bazel` file. For example:

```python
register_toolchains("//toolchains:buffa_toolchain")
```

#### <a name="rust_buffa_transform"></a>Transforming generated crates

`rust_buffa_transform` adjusts the crate a single `proto_library` generates: a
crate-name override, extra codegen options, extra crate features, extra Rust
dependencies, and extra source files appended to the crate root. Attach it
through the `proto_library`'s `data` attribute:

```python
rust_buffa_transform(
    name = "my_transform",
    opts = ["json=true"],
    deps = ["@crates//:serde"],
)

proto_library(
    name = "my_proto",
    srcs = ["my.proto"],
    data = [":my_transform"],
)
```

Options that change what other crates see (e.g. `views=false`) must agree
with the options dependents were generated with.

#### <a name="services"></a>Services

Service stubs are generated through [connectrpc][connectrpc] when the bundled
connect toolchain is registered; it is not registered by default, so crates
stay message-only unless a workspace opts in. buffa generates message types
only; `connectrpc` is the service layer built on it — its `connectrpc-codegen`
generator wraps `buffa-codegen`, and its runtime serves the Connect, gRPC,
and gRPC-Web protocols.

[connectrpc]: https://github.com/connectrpc/connect-rust

```python
register_toolchains("@rules_buffa//:connect_buffa_toolchain")
```

Every `proto_library` crate then also gets a `<Service>` trait and
`<SERVICE>_SERVICE_NAME` constant per `service` declaration, plus JSON and
view impls, which the Connect codec and stubs require even when `json` and
`views` are not set in `opts`. Three extra option keys are accepted only by
this toolchain's codegen:

- `gate_client_feature` (bool): `#[cfg(feature = ...)]`-gate the generated
  client structs so server-only deployments can drop the client transport.
- `client_feature_name=<name>`: the crate feature that gates the clients
  (default `client`); add it to `crate_features` to enable clients.
- `encodable_impls=outputs|all_messages`: emit `Encodable` view impls for
  every message in the crate, not only RPC output types, when other crates'
  services serve this crate's messages.

To build the connect codegen against your own crates, compile the same
exported source with the `connect` crate feature and add `connectrpc-codegen`:

```python
rust_binary(
    name = "connect_codegen",
    srcs = ["@rules_buffa//private:codegen.rs"],
    crate_features = ["connect"],
    edition = "2021",
    deps = [
        "@crates//:buffa",
        "@crates//:buffa-codegen",
        "@crates//:connectrpc-codegen",
    ],
)
```

and point a `rust_buffa_toolchain`'s `codegen` at it, with `connectrpc` and
`serde` (for the JSON impls) in `runtime`.

#### Differences from `rules_rust_prost`

- Module and type names follow buffa: proto package casing is kept
  (`package Foo.Bar` → `Foo::Bar`), nested types live in a snake_case module
  named after the message, and field names are verbatim unless
  `idiomatic_field_names=true`.
- Well-known types are not generated: `google.protobuf` types come from
  `buffa-types`, and `descriptor.proto`/`plugin.proto` types from
  `buffa-descriptor`.
- `#[derive(Eq)]` through `type_attribute` compiles only with
  `unknown_fields=false`, because buffa's unknown-field storage is not `Eq`.
- Service stubs come from ConnectRPC rather than tonic; see
  [Services](#services).

---
"""

load(
    "//private:buffa.bzl",
    _rust_buffa_library = "rust_buffa_library",
    _rust_buffa_toolchain = "rust_buffa_toolchain",
    _rust_buffa_transform = "rust_buffa_transform",
)

rust_buffa_library = _rust_buffa_library
rust_buffa_toolchain = _rust_buffa_toolchain
rust_buffa_transform = _rust_buffa_transform
