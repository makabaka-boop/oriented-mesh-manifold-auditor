//! 命令行端到端测试：退出码 0/1/2 与输出格式。

use std::io::Write;
use std::process::{Command, Stdio};

fn run_cli(input: &str) -> (i32, String, String) {
    let exe = env!("CARGO_BIN_EXE_meshcheck");
    let mut child = Command::new(exe)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn meshcheck");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

const TETRA: &str = "\
# closed tetrahedron
v 0
v 1
v 2
v 3
f 0 2 1
f 0 1 3
f 0 3 2
f 1 2 3
";

#[test]
fn ok_mesh_exits_zero_with_invariants() {
    let (code, stdout, stderr) = run_cli(TETRA);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "OK V=4 E=6 F=4 euler=2 genus=0");
    assert!(stderr.is_empty());
}

#[test]
fn flipped_face_exits_one_with_edge_witness() {
    let input = TETRA.replace("f 0 2 1", "f 0 1 2");
    let (code, stdout, _) = run_cli(&input);
    assert_eq!(code, 1);
    assert_eq!(
        stdout.trim(),
        "FAIL edge orientation 0-1 witness=edge(0,1)"
    );
}

#[test]
fn pinched_vertex_exits_one_with_vertex_witness() {
    let input = "\
v 0
v 1
v 2
v 3
v 4
v 5
v 6
f 0 2 1
f 0 1 3
f 0 3 2
f 1 2 3
f 0 5 4
f 0 4 6
f 0 6 5
f 4 5 6
";
    let (code, stdout, _) = run_cli(input);
    assert_eq!(code, 1);
    assert_eq!(
        stdout.trim(),
        "FAIL vertex non-manifold 0 sectors=2 witness=vertex(0)"
    );
}

#[test]
fn disconnected_shells_exit_one_with_face_witness() {
    let input = "\
v 0
v 1
v 2
v 3
v 4
v 5
v 6
v 7
f 0 2 1
f 0 1 3
f 0 3 2
f 1 2 3
f 4 6 5
f 4 5 7
f 4 7 6
f 5 6 7
";
    let (code, stdout, _) = run_cli(input);
    assert_eq!(code, 1);
    assert_eq!(
        stdout.trim(),
        "FAIL connectivity face_index=4 witness=face#4"
    );
}

#[test]
fn unknown_vertex_is_rejected() {
    let input = TETRA.replace("f 1 2 3", "f 1 2 9");
    let (code, _, stderr) = run_cli(&input);
    assert_eq!(code, 2);
    assert!(stderr.contains("REJECT"), "stderr: {stderr}");
    assert!(stderr.contains("unknown vertex"), "stderr: {stderr}");
}

#[test]
fn duplicate_undirected_face_is_rejected() {
    // f 3 2 1 与 f 1 2 3 是同一无向面。
    let input = TETRA.replace("f 1 2 3", "f 1 2 3\nf 3 2 1");
    let (code, _, stderr) = run_cli(&input);
    assert_eq!(code, 2);
    assert!(stderr.contains("duplicate undirected face"), "stderr: {stderr}");
}

#[test]
fn extra_field_is_rejected() {
    let input = TETRA.replace("f 1 2 3", "f 1 2 3 4");
    let (code, _, stderr) = run_cli(&input);
    assert_eq!(code, 2);
    assert!(stderr.contains("expects exactly 4 fields"), "stderr: {stderr}");
}

#[test]
fn degenerate_face_is_rejected() {
    let input = TETRA.replace("f 1 2 3", "f 1 2 2");
    let (code, _, stderr) = run_cli(&input);
    assert_eq!(code, 2);
    assert!(stderr.contains("pairwise distinct"), "stderr: {stderr}");
}

#[test]
fn duplicate_vertex_is_rejected() {
    let input = TETRA.replace("v 3", "v 3\nv 3");
    let (code, _, stderr) = run_cli(&input);
    assert_eq!(code, 2);
    assert!(stderr.contains("duplicate vertex"), "stderr: {stderr}");
}

#[test]
fn too_few_vertices_is_rejected() {
    let input = "\
v 0
v 1
v 2
f 0 1 2
";
    let (code, _, stderr) = run_cli(input);
    assert_eq!(code, 2);
    assert!(stderr.contains("vertex count"), "stderr: {stderr}");
}

#[test]
fn non_ascii_is_rejected() {
    let input = format!("{TETRA}v 5\u{e9}\n");
    let (code, _, stderr) = run_cli(&input);
    assert_eq!(code, 2);
    assert!(stderr.contains("ASCII"), "stderr: {stderr}");
}

#[test]
fn torus_file_reports_genus_one() {
    let exe = env!("CARGO_BIN_EXE_meshcheck");
    let out = Command::new(exe)
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/data/torus.mesh"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.trim(), "OK V=15 E=45 F=30 euler=0 genus=1");
}
