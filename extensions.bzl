"""Bzlmod module extensions"""

load("//private/3rdparty/crates:crates.bzl", "crate_repositories")

def _rust_ext_impl(module_ctx):
    direct_deps = []

    direct_deps.extend(crate_repositories())

    # is_dev_dep is ignored here. It's not relevant for internal_deps, as dev
    # dependencies are only relevant for module extensions that can be used
    # by other MODULES.
    return module_ctx.extension_metadata(
        root_module_direct_deps = [repo.repo for repo in direct_deps],
        root_module_direct_dev_deps = [],
    )

rust_ext = module_extension(
    doc = "Dependencies for rules_buffa.",
    implementation = _rust_ext_impl,
)
