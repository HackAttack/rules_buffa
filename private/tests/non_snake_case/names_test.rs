//! `non_snake_case` leaks rustc denies: the crate name comes from the
//! `Lint_Names_proto` target name and the proto field keeps its `__` name.

#[test]
fn pose_field() {
    let pose = Lint_Names_proto::lint::names::Pose {
        pos_vcf_odom__odom: 3,
        ..Default::default()
    };
    assert_eq!(3, pose.pos_vcf_odom__odom);
}

#[test]
fn buffa_srcs_output_group() {
    let r = runfiles::Runfiles::create().unwrap();
    let path = runfiles::rlocation!(r, std::env::var("BUFFA_SRC").unwrap()).unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    assert!(src.contains("pub struct Pose"));
}
