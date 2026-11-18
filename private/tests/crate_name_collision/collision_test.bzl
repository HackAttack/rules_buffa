"""Tests that same-named `proto_library` crates fail at analysis, not rustc."""

load("@bazel_skylib//lib:unittest.bzl", "analysistest", "asserts")

def _collision_test_impl(ctx):
    env = analysistest.begin(ctx)
    asserts.expect_failure(env, "both generate a crate named `same_proto`")
    return analysistest.end(env)

collision_test = analysistest.make(
    _collision_test_impl,
    expect_failure = True,
)
