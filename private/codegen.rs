//! Generates a single-file Rust crate from `proto_library` descriptor sets
//! using `buffa_codegen`, plus an extern-path manifest that lets dependent
//! crates reference these types instead of regenerating them.
//!
//! argv: `<crate_name> <lib_rs> <manifest> <descriptor_set> <deps_file> <sets_file> <srcs_file> <reexports_file> [opt...]`
//! where `deps_file`, `sets_file`, `srcs_file`, and `reexports_file` list,
//! one entry per line, the dependency crates' manifests, the transitive
//! `FileDescriptorSet`s, extra sources appended to the crate root, and the
//! dep crate names re-exported from the crate root respectively, and each
//! trailing `opt` is a `key=value` codegen option.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

use buffa_codegen::context::CodeGenContext;
#[cfg(not(feature = "connect"))]
use buffa_codegen::generate_with_diagnostics;
use buffa_codegen::generated::descriptor::FileDescriptorSet;
use buffa_codegen::{
    generate_module_tree, BytesRepr, CodeGenConfig, EnumTypeOverride, FeatureOverride,
    GeneratedFileKind, IncludeMode, MapRepr, PointerRepr, ReflectMode, RepeatedRepr, StringRepr,
};
#[cfg(feature = "connect")]
use connectrpc_codegen::codegen::{EncodableImpls, Options as ConnectOptions};

/// The `proto` files whose types `buffa-types` and `buffa-descriptor` already
/// provide. A `proto_library` of only these generates an empty crate and an
/// empty manifest; dependents resolve them through buffa's built-in
/// `.google.protobuf` extern mapping.
const PROVIDED_WKT_FILES: &[&str] = &[
    "google/protobuf/any.proto",
    "google/protobuf/api.proto",
    "google/protobuf/compiler/plugin.proto",
    "google/protobuf/descriptor.proto",
    "google/protobuf/duration.proto",
    "google/protobuf/empty.proto",
    "google/protobuf/field_mask.proto",
    "google/protobuf/source_context.proto",
    "google/protobuf/struct.proto",
    "google/protobuf/timestamp.proto",
    "google/protobuf/type.proto",
    "google/protobuf/wrappers.proto",
];

fn read_fds(path: &str) -> Result<FileDescriptorSet, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    buffa_codegen::tooling_decode_options()?
        .decode_from_slice::<FileDescriptorSet>(&bytes)
        .map_err(|e| format!("decode FileDescriptorSet {path}: {e}"))
}

/// Reads a file of newline-separated paths and returns the paths.
fn read_path_list(path: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    Ok(content
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn parse_bool(opt: &str, value: &str) -> Result<bool, String> {
    value
        .parse::<bool>()
        .map_err(|_| format!("invalid codegen option {opt:?}: expected a bool value"))
}

/// Normalizes a proto path to buffa's prefix-matched form: `.` matches
/// everything, a leading dot is added if missing, and trailing dots are
/// stripped.
fn normalize_proto_path(opt: &str, path: &str) -> Result<String, String> {
    let empty = || {
        format!("invalid codegen option {opt:?}: empty proto path (use `.` to match everything)")
    };
    if path.is_empty() {
        return Err(empty());
    }
    if path == "." {
        return Ok(".".to_string());
    }
    let mut path = path.to_string();
    if !path.starts_with('.') {
        path.insert(0, '.');
    }
    while path.ends_with('.') {
        path.pop();
    }
    if path.is_empty() {
        return Err(empty());
    }
    Ok(path)
}

/// Applies one `key=value` codegen option to `config`. Options apply in
/// order; for path-scoped rules the last matching rule wins.
fn apply_opt(config: &mut CodeGenConfig, opt: &str) -> Result<(), String> {
    let (key, value) = opt
        .split_once('=')
        .ok_or_else(|| format!("invalid codegen option {opt:?}: expected `key=value`"))?;
    // buffa's own option parser is private to protoc-gen-buffa and lacks the type and attribute settings, so keys are mapped here.
    match key {
        "views" => config.generate_views = parse_bool(opt, value)?,
        "lazy_views" => config.lazy_views = parse_bool(opt, value)?,
        "unknown_fields" => config.preserve_unknown_fields = parse_bool(opt, value)?,
        "json" => config.generate_json = parse_bool(opt, value)?,
        "text" => config.generate_text = parse_bool(opt, value)?,
        "arbitrary" => config.generate_arbitrary = parse_bool(opt, value)?,
        "allow_message_set" => config.allow_message_set = parse_bool(opt, value)?,
        "strict_utf8_mapping" => config.strict_utf8_mapping = parse_bool(opt, value)?,
        "register_types" => config.emit_register_fn = parse_bool(opt, value)?,
        "with_setters" => config.generate_with_setters = parse_bool(opt, value)?,
        "idiomatic_imports" => config.idiomatic_imports = parse_bool(opt, value)?,
        "idiomatic_field_names" => config.idiomatic_field_names = parse_bool(opt, value)?,
        "idiomatic_enum_aliases" => config.idiomatic_enum_aliases = parse_bool(opt, value)?,
        "gate_impls" => config.gate_impls_on_crate_features = parse_bool(opt, value)?,
        "shared_descriptor_pool" => config.shared_descriptor_pool = parse_bool(opt, value)?,
        key @ ("json_feature" | "views_feature" | "text_feature" | "reflect_feature") => {
            if value.is_empty() {
                return Err(format!(
                    "invalid codegen option {opt:?}: empty feature name"
                ));
            }
            let names = &mut config.feature_gate_names;
            match key {
                "json_feature" => names.json = value.to_string(),
                "views_feature" => names.views = value.to_string(),
                "text_feature" => names.text = value.to_string(),
                _ => names.reflect = value.to_string(),
            }
        }
        "extern_path" => {
            // `<proto package or type>=<rust path>`; both halves are required.
            let (proto, rust) = value.split_once('=').ok_or_else(|| {
                format!(
                    "invalid codegen option {opt:?}: expected `extern_path=<proto_path>=<rust_path>`"
                )
            })?;
            if proto.is_empty() || rust.is_empty() {
                return Err(format!(
                    "invalid codegen option {opt:?}: expected `extern_path=<proto_path>=<rust_path>`"
                ));
            }
            config
                .extern_paths
                .push((normalize_proto_path(opt, proto)?, rust.to_string()));
        }
        "unbox_oneof" => {
            if parse_bool(opt, value)? {
                config.unboxed_oneof_fields.push(".".to_string());
            }
        }
        "unbox_oneof_in" => config
            .unboxed_oneof_fields
            .push(normalize_proto_path(opt, value)?),
        "reflection" => {
            let mode = if parse_bool(opt, value)? {
                ReflectMode::VTable
            } else {
                ReflectMode::Off
            };
            mode.apply(config);
        }
        "reflect_mode" => match value {
            "off" => ReflectMode::Off.apply(config),
            "bridge" => ReflectMode::Bridge.apply(config),
            "vtable" => ReflectMode::VTable.apply(config),
            _ => {
                return Err(format!(
                    "invalid codegen option {opt:?}: expected `off`, `bridge`, or `vtable`"
                ))
            }
        },
        "type_name_prefix" => config.type_name_prefix = value.to_string(),
        "open_enums_in" => config.feature_overrides.push((
            normalize_proto_path(opt, value)?,
            FeatureOverride::EnumType(EnumTypeOverride::Open),
        )),
        "override_feature_in" => {
            // `<proto_path>=<feature>:<value>`; `enum_type:open` is the only
            // supported override (the `open_enums_in` shorthand maps to it).
            let (path, spec) = value.split_once('=').ok_or_else(|| {
                format!("invalid codegen option {opt:?}: expected `override_feature_in=<proto_path>=<feature>:<value>`")
            })?;
            let (feature, feature_value) = spec.split_once(':').ok_or_else(|| {
                format!("invalid codegen option {opt:?}: expected `override_feature_in=<proto_path>=<feature>:<value>`")
            })?;
            if feature != "enum_type" || !feature_value.eq_ignore_ascii_case("open") {
                return Err(format!(
                    "unsupported codegen option {opt:?}: supported overrides: `enum_type:open`"
                ));
            }
            config.feature_overrides.push((
                normalize_proto_path(opt, path)?,
                FeatureOverride::EnumType(EnumTypeOverride::Open),
            ));
        }
        _ => apply_typed_opt(config, opt, key, value)?,
    }
    Ok(())
}

/// Applies the path-scoped option families: the five `*_attribute` keys and
/// the `<kind>[=_custom][_in]` field-representation keys.
fn apply_typed_opt(
    config: &mut CodeGenConfig,
    opt: &str,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let attributes = match key {
        "type_attribute" => Some(&mut config.type_attributes),
        "field_attribute" => Some(&mut config.field_attributes),
        "message_attribute" => Some(&mut config.message_attributes),
        "enum_attribute" => Some(&mut config.enum_attributes),
        "oneof_attribute" => Some(&mut config.oneof_attributes),
        _ => None,
    };
    if let Some(attributes) = attributes {
        // The attribute itself may contain `=` and `,`; split the value once.
        let (path, attribute) = value.split_once('=').ok_or_else(|| {
            format!("invalid codegen option {opt:?}: expected `{key}=<proto_path>=<attribute>`")
        })?;
        attributes.push((normalize_proto_path(opt, path)?, attribute.to_string()));
        return Ok(());
    }

    // Strip `_in` (the rule names a proto path) then `_custom` (the repr is a
    // Rust path or template) to find the kind.
    let mut kind = key;
    let scoped = kind.ends_with("_in");
    if scoped {
        kind = &kind[..kind.len() - "_in".len()];
    }
    let custom = kind.ends_with("_custom");
    if custom {
        kind = &kind[..kind.len() - "_custom".len()];
    }
    if !matches!(
        kind,
        "bytes_type" | "string_type" | "map_type" | "box_type" | "repeated_type"
    ) {
        return Err(format!("unsupported codegen option {opt:?}"));
    }
    let (path, repr) = if scoped {
        let (path, repr) = value.split_once('=').ok_or_else(|| {
            format!("invalid codegen option {opt:?}: expected `{key}=<proto_path>=<repr>`")
        })?;
        (normalize_proto_path(opt, path)?, repr)
    } else {
        (".".to_string(), value)
    };
    macro_rules! repr {
        ($rules:expr, $ty:ident, $($name:literal => $variant:ident),+ $(,)?) => {
            $rules.push((
                path,
                if custom {
                    $ty::Custom(repr.to_string())
                } else {
                    match repr {
                        $($name => $ty::$variant,)+
                        other => {
                            return Err(format!(
                                "invalid codegen option {opt:?}: unknown repr {other:?}, expected {}",
                                [$($name),+].join(" or "),
                            ))
                        }
                    }
                },
            ))
        };
    }
    match kind {
        "bytes_type" => repr!(config.bytes_fields, BytesRepr, "vec" => Vec, "bytes" => Bytes),
        "string_type" => repr!(config.string_fields, StringRepr, "string" => String),
        "map_type" => repr!(
            config.map_fields,
            MapRepr,
            "hash_map" => HashMap,
            "btree_map" => BTreeMap
        ),
        "box_type" => repr!(config.pointer_fields, PointerRepr, "box" => Box, "inline" => Inline),
        "repeated_type" => repr!(config.repeated_fields, RepeatedRepr, "vec" => Vec),
        _ => unreachable!("kind was validated above"),
    }
    Ok(())
}

/// Applies one connectrpc service-stub option and reports whether `opt` was
/// one. Unknown keys return `Ok(false)` so the caller falls through to the
/// ordinary `apply_opt` (and from there to `unsupported codegen option`).
#[cfg(feature = "connect")]
fn apply_connect_opt(options: &mut ConnectOptions, opt: &str) -> Result<bool, String> {
    let (key, value) = opt
        .split_once('=')
        .ok_or_else(|| format!("invalid codegen option {opt:?}: expected `key=value`"))?;
    match key {
        "gate_client_feature" => options.gate_client_feature = parse_bool(opt, value)?,
        "client_feature_name" => {
            if value.is_empty() {
                return Err(format!(
                    "invalid codegen option {opt:?}: empty client feature name"
                ));
            }
            options.client_feature_name = value.to_string();
        }
        "encodable_impls" => {
            options.encodable_impls = match value {
                "outputs" => EncodableImpls::Outputs,
                "all_messages" => EncodableImpls::AllMessages,
                _ => {
                    return Err(format!(
                        "invalid codegen option {opt:?}: expected `encodable_impls=outputs` or `encodable_impls=all_messages`"
                    ))
                }
            };
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 9 {
        return Err(format!(
            "usage: {} <crate_name> <lib_rs> <manifest> <descriptor_set> <deps_file> <sets_file> <srcs_file> <reexports_file> [opt...]",
            args[0]
        ));
    }
    let crate_name = &args[1];

    // Files this crate generates: everything in the direct descriptor set
    // except the well-known types provided by the runtime crates.
    let direct = read_fds(&args[4])?;
    let files_to_generate: Vec<String> = direct
        .file
        .iter()
        .filter_map(|file| file.name.clone())
        .filter(|name| !PROVIDED_WKT_FILES.contains(&name.as_str()))
        .collect();

    // Merge every transitive descriptor set, deduplicated by proto file name.
    let mut all_files = Vec::new();
    let mut seen = HashSet::new();
    for path in read_path_list(&args[6])? {
        for file in read_fds(&path)?.file {
            match &file.name {
                Some(name) if seen.insert(name.clone()) => all_files.push(file),
                _ => {}
            }
        }
    }

    let mut config = CodeGenConfig::default();
    config.file_per_package = true;
    #[cfg(feature = "connect")]
    {
        // The Connect JSON codec needs the JSON impls; `json=false` can
        // still override.
        config.generate_json = true;
    }
    #[cfg(feature = "connect")]
    let mut connect_options = ConnectOptions::default();
    for opt in &args[9..] {
        #[cfg(feature = "connect")]
        if apply_connect_opt(&mut connect_options, opt)? {
            continue;
        }
        apply_opt(&mut config, opt)?;
    }
    // Dependency manifests map proto type FQNs to absolute Rust paths so buffa
    // references the owning crate instead of regenerating the types. User
    // `extern_path` options were applied above, and buffa resolves exact
    // matches in order, so a user mapping wins over a manifest entry.
    for dep_path in read_path_list(&args[5])? {
        for line in read_path_list(&dep_path)? {
            let (proto_fqn, rust_path) = line
                .split_once('=')
                .ok_or_else(|| format!("malformed extern path in {dep_path}: {line}"))?;
            config
                .extern_paths
                .push((proto_fqn.to_string(), rust_path.to_string()));
        }
    }

    #[cfg(feature = "connect")]
    let files = {
        // Service stubs reference view types, so connectrpc forces views
        // on; keep `config` in sync so the extern-path manifest's
        // `CodeGenContext` below matches what was generated.
        config.generate_views = true;
        connect_options.buffa = config.clone();
        connectrpc_codegen::codegen::generate_files(
            &all_files,
            &files_to_generate,
            &connect_options,
        )
        .map_err(|e| format!("{e:#}"))?
    };
    #[cfg(not(feature = "connect"))]
    let files = {
        let (files, warnings) = generate_with_diagnostics(&all_files, &files_to_generate, &config)
            .map_err(|e| e.to_string())?;
        for warning in &warnings {
            eprintln!("warning: {warning}");
        }
        files
    };

    // With `file_per_package` only `PackageMod` files are emitted; inline each
    // into the module tree to produce one crate-root source file.
    let package_mods: Vec<_> = files
        .iter()
        .filter(|file| file.kind == GeneratedFileKind::PackageMod)
        .collect();
    let entries: Vec<(&str, &str)> = package_mods
        .iter()
        .map(|file| (file.name.as_str(), file.package.as_str()))
        .collect();
    let mut lib_rs = generate_module_tree(&entries, IncludeMode::Relative(""), true);
    // buffa keeps proto casing in module names, crate names come from target
    // names, and buffa's own allow misses lowercase names containing `__`.
    lib_rs.insert_str(0, "#![allow(non_snake_case)]\n");
    for file in &package_mods {
        let include = format!("include!(\"{}\");", file.name);
        assert!(
            lib_rs.contains(&include),
            "module tree emitted no {include}"
        );
        lib_rs = lib_rs.replacen(&include, &file.content, 1);
    }

    // Re-export the dep crates this crate compiles against so dependents can
    // reach them as `<crate>::<dep_crate>::...`.
    for name in read_path_list(&args[8])? {
        lib_rs.push_str(&format!("pub use ::{name};\n"));
    }

    // With a shared descriptor pool, `generate` emits per-package
    // `__buffa::reflect` modules that delegate to a `__buffa_fds` module at
    // the module-tree root — which the crate root is here (mirrors
    // buffa-build's tree assembly).
    if config.shared_descriptor_pool && !files_to_generate.is_empty() {
        lib_rs.push_str(&buffa_codegen::shared_descriptor_root_module(
            &buffa_codegen::encode_descriptor_set(&all_files, &config.feature_overrides),
            buffa_codegen::FdsEmbedding::Inline,
            config.reflect_feature_gate(),
        ));
        lib_rs.push('\n');
    }

    // `rust_buffa_transform` sources are appended verbatim to the crate root.
    for path in read_path_list(&args[7])? {
        lib_rs.push_str(&fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?);
        lib_rs.push('\n');
    }
    fs::write(&args[2], lib_rs).map_err(|e| format!("write {}: {e}", args[2]))?;

    // The manifest records the types this crate owns: every type whose Rust
    // path stayed local (extern-mapped types are absolute paths).
    let context = CodeGenContext::for_generate(&all_files, &files_to_generate, &config);
    let mut lines: Vec<String> = context
        .type_map
        .iter()
        .filter(|(_, rust_path)| !rust_path.starts_with("::") && !rust_path.starts_with("crate::"))
        .map(|(proto_fqn, rust_path)| format!("{proto_fqn}=::{crate_name}::{rust_path}"))
        .collect();
    lines.sort();
    fs::write(&args[3], lines.join("\n")).map_err(|e| format!("write {}: {e}", args[3]))?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(opt: &str) -> CodeGenConfig {
        let mut config = CodeGenConfig::default();
        apply_opt(&mut config, opt).unwrap();
        config
    }

    type BoolField = fn(&CodeGenConfig) -> bool;

    #[test]
    fn bool_options_flip_their_field() {
        let cases: &[(&str, BoolField)] = &[
            ("views", |c| c.generate_views),
            ("lazy_views", |c| c.lazy_views),
            ("unknown_fields", |c| c.preserve_unknown_fields),
            ("json", |c| c.generate_json),
            ("text", |c| c.generate_text),
            ("arbitrary", |c| c.generate_arbitrary),
            ("allow_message_set", |c| c.allow_message_set),
            ("strict_utf8_mapping", |c| c.strict_utf8_mapping),
            ("register_types", |c| c.emit_register_fn),
            ("with_setters", |c| c.generate_with_setters),
            ("idiomatic_imports", |c| c.idiomatic_imports),
            ("idiomatic_field_names", |c| c.idiomatic_field_names),
            ("idiomatic_enum_aliases", |c| c.idiomatic_enum_aliases),
            ("gate_impls", |c| c.gate_impls_on_crate_features),
            ("shared_descriptor_pool", |c| c.shared_descriptor_pool),
        ];
        for (key, get) in cases {
            let mut config = CodeGenConfig::default();
            let flipped = !get(&config);
            apply_opt(&mut config, &format!("{key}={flipped}")).unwrap();
            assert_eq!(get(&config), flipped, "{key}");
        }
    }

    #[test]
    fn unbox_oneof_pushes_normalized_paths() {
        let mut config = config_with("unbox_oneof=true");
        assert_eq!(config.unboxed_oneof_fields, ["."]);
        apply_opt(&mut config, "unbox_oneof_in=my.pkg.Msg.").unwrap();
        assert_eq!(config.unboxed_oneof_fields, [".", ".my.pkg.Msg"]);
        apply_opt(&mut config, "unbox_oneof=false").unwrap();
        assert_eq!(config.unboxed_oneof_fields, [".", ".my.pkg.Msg"]);
    }

    #[test]
    fn reflection_options_select_modes() {
        let config = config_with("reflection=true");
        assert!(config.generate_reflection && config.generate_reflection_vtable);
        let config = config_with("reflect_mode=bridge");
        assert!(config.generate_reflection && !config.generate_reflection_vtable);
        let config = config_with("reflect_mode=off");
        assert!(!config.generate_reflection && !config.generate_reflection_vtable);
    }

    #[test]
    fn type_name_prefix_is_verbatim() {
        assert_eq!(config_with("type_name_prefix=Rpc").type_name_prefix, "Rpc");
    }

    #[test]
    fn open_enums_in_pushes_enum_type_open() {
        let config = config_with("open_enums_in=my.pkg");
        assert_eq!(
            config.feature_overrides,
            [(
                ".my.pkg".to_string(),
                FeatureOverride::EnumType(EnumTypeOverride::Open)
            )]
        );
    }

    #[test]
    fn override_feature_in_accepts_only_enum_type_open() {
        let config = config_with("override_feature_in=my.pkg.E=enum_type:open");
        assert_eq!(
            config.feature_overrides,
            [(
                ".my.pkg.E".to_string(),
                FeatureOverride::EnumType(EnumTypeOverride::Open)
            )]
        );
        for opt in [
            "override_feature_in=.my.pkg.E=enum_type:closed",
            "override_feature_in=.my.pkg.E=message_encoding:DELIMITED",
            "override_feature_in=.my.pkg.E",
        ] {
            let err = apply_opt(&mut CodeGenConfig::default(), opt).unwrap_err();
            assert!(
                err.contains("enum_type:open") || err.contains("expected"),
                "{opt}: {err}"
            );
        }
    }

    #[test]
    fn extern_path_pushes_a_mapping() {
        let config = config_with("extern_path=.a=::b::c");
        assert_eq!(
            config.extern_paths,
            [(".a".to_string(), "::b::c".to_string())]
        );
    }

    #[test]
    fn feature_gate_names_set_and_reject_empty() {
        let config = config_with("json_feature=serde");
        assert_eq!(config.feature_gate_names.json, "serde");
        assert_eq!(config.feature_gate_names.views, "views");
        assert!(apply_opt(&mut CodeGenConfig::default(), "json_feature=").is_err());
        assert!(apply_opt(&mut CodeGenConfig::default(), "reflect_feature=").is_err());
    }

    #[test]
    fn unknown_scoped_keys_are_unsupported() {
        for opt in ["bogus_in=x", "bogus_custom_in=x"] {
            let err = apply_opt(&mut CodeGenConfig::default(), opt).unwrap_err();
            assert!(err.contains("unsupported"), "{opt}: {err}");
        }
    }

    #[test]
    fn field_repr_options_push_rules() {
        let config = config_with("bytes_type=bytes");
        assert_eq!(config.bytes_fields, [(".".to_string(), BytesRepr::Bytes)]);

        let config = config_with("string_type_custom=::compact_str::CompactString");
        assert_eq!(
            config.string_fields,
            [(
                ".".to_string(),
                StringRepr::Custom("::compact_str::CompactString".to_string())
            )]
        );

        let config = config_with("map_type_in=.my.pkg=btree_map");
        assert_eq!(
            config.map_fields,
            [(".my.pkg".to_string(), MapRepr::BTreeMap)]
        );

        let config = config_with("box_type_custom_in=my.pkg.Msg.field=::my::Ptr");
        assert_eq!(
            config.pointer_fields,
            [(
                ".my.pkg.Msg.field".to_string(),
                PointerRepr::Custom("::my::Ptr".to_string())
            )]
        );

        let config = config_with("repeated_type=vec");
        assert_eq!(
            config.repeated_fields,
            [(".".to_string(), RepeatedRepr::Vec)]
        );
    }

    #[test]
    fn attribute_options_keep_the_attribute_verbatim() {
        let config = config_with(r#"field_attribute=my.pkg.Msg.f=#[serde(rename = "a,b")]"#);
        assert_eq!(
            config.field_attributes,
            [(
                ".my.pkg.Msg.f".to_string(),
                r#"#[serde(rename = "a,b")]"#.to_string()
            )]
        );
    }

    #[test]
    fn invalid_options_error() {
        for opt in [
            "bogus=true",
            "json",
            "json=maybe",
            "bytes_type=nope",
            "map_type_in=btree_map",
            "unbox_oneof_in=",
            "extern_path=.a",
        ] {
            assert!(
                apply_opt(&mut CodeGenConfig::default(), opt).is_err(),
                "{opt}"
            );
        }
    }

    #[cfg(feature = "connect")]
    mod connect {
        use super::*;

        #[test]
        fn connect_options_apply() {
            let mut options = ConnectOptions::default();
            assert_eq!(
                apply_connect_opt(&mut options, "gate_client_feature=true"),
                Ok(true)
            );
            assert_eq!(
                apply_connect_opt(&mut options, "client_feature_name=grpc_client"),
                Ok(true)
            );
            assert_eq!(
                apply_connect_opt(&mut options, "encodable_impls=all_messages"),
                Ok(true)
            );
            assert!(options.gate_client_feature);
            assert_eq!(options.client_feature_name, "grpc_client");
            assert_eq!(options.encodable_impls, EncodableImpls::AllMessages);
            assert_eq!(
                apply_connect_opt(&mut options, "encodable_impls=outputs"),
                Ok(true)
            );
            assert_eq!(options.encodable_impls, EncodableImpls::Outputs);
        }

        #[test]
        fn connect_option_errors() {
            let mut options = ConnectOptions::default();
            for opt in [
                "gate_client_feature",
                "gate_client_feature=yes",
                "client_feature_name",
                "client_feature_name=",
                "encodable_impls=",
                "encodable_impls=some",
            ] {
                assert!(
                    apply_connect_opt(&mut options, opt).is_err(),
                    "expected {opt:?} to fail"
                );
            }
        }

        #[test]
        fn non_connect_opts_pass_through() {
            let mut options = ConnectOptions::default();
            assert_eq!(apply_connect_opt(&mut options, "json=true"), Ok(false));
            assert_eq!(apply_connect_opt(&mut options, "bogus=1"), Ok(false));
        }
    }
}
