//! 拓扑审计集成测试：翻面、边界边、非流形边、共享单顶点的双壳、
//! 不连通双壳、合法封闭体（球面与亏格 1 环面）。

use meshcheck::{audit, parse, Audit, EdgeAnomaly, EdgeKind, Finding};

fn run(input: &str) -> Audit {
    let mesh = parse(input).expect("test mesh must parse");
    audit(&mesh)
}

/// 合法封闭四面体：V=4 E=6 F=4，χ=2，亏格 0。
const TETRA: &str = "\
v 0
v 1
v 2
v 3
f 0 2 1
f 0 1 3
f 0 3 2
f 1 2 3
";

/// 翻面：把四面体的 f 0 2 1 反向为 f 0 1 2，
/// 边 {0,1}、{0,2}、{1,2} 全部被两个面同向使用。
const TETRA_FLIPPED: &str = "\
v 0
v 1
v 2
v 3
f 0 1 2
f 0 1 3
f 0 3 2
f 1 2 3
";

/// 三角双锥（合法封闭体，V=5 E=9 F=6）去掉顶面 f 0 4 2，
/// 留下边界边 {0,2}、{0,4}、{2,4}，最小见证为 (0,2)。
const BIPYRAMID_OPEN: &str = "\
v 0
v 1
v 2
v 3
v 4
f 0 2 3
f 0 3 4
f 1 3 2
f 1 4 3
f 1 2 4
";

/// 两个四面体只共享顶点 0：顶点 0 处捏成两个互不相连的扇区。
const PINCHED_VERTEX: &str = "\
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

/// 两个互不相交的四面体：边与顶点检查都通过，但整体不连通。
const TWO_SHELLS: &str = "\
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

/// 三个面共用无向边 {0,1}（非流形边），其余边均为边界边；
/// 最小异常见证仍是非流形边 {0,1}。
const NON_MANIFOLD_EDGE: &str = "\
v 0
v 1
v 2
v 3
v 4
f 0 1 2
f 1 0 3
f 0 1 4
f 2 3 4
";

#[test]
fn closed_tetrahedron_is_genus_zero() {
    assert_eq!(
        run(TETRA),
        Audit::Ok {
            vertices: 4,
            edges: 6,
            faces: 4,
            euler: 2,
            genus: 0,
        }
    );
}

#[test]
fn flipped_face_reports_smallest_orientation_edge() {
    assert_eq!(
        run(TETRA_FLIPPED),
        Audit::Failed(Finding::Edge(EdgeAnomaly {
            kind: EdgeKind::Orientation,
            lo: 0,
            hi: 1,
        }))
    );
}

#[test]
fn open_shell_reports_smallest_boundary_edge() {
    assert_eq!(
        run(BIPYRAMID_OPEN),
        Audit::Failed(Finding::Edge(EdgeAnomaly {
            kind: EdgeKind::Boundary,
            lo: 0,
            hi: 2,
        }))
    );
}

#[test]
fn triple_used_edge_reports_non_manifold_edge() {
    assert_eq!(
        run(NON_MANIFOLD_EDGE),
        Audit::Failed(Finding::Edge(EdgeAnomaly {
            kind: EdgeKind::NonManifold,
            lo: 0,
            hi: 1,
        }))
    );
}

#[test]
fn two_shells_sharing_one_vertex_report_pinched_vertex() {
    // 每条边都恰被两个面反向使用，但顶点 0 处扇区分裂为二。
    assert_eq!(
        run(PINCHED_VERTEX),
        Audit::Failed(Finding::Vertex { id: 0, sectors: 2 })
    );
}

#[test]
fn disjoint_shells_report_smallest_detached_face() {
    // 顶点 4..7 构成的壳与面 0 不连通，最小脱离面下标为 4。
    assert_eq!(
        run(TWO_SHELLS),
        Audit::Failed(Finding::Connectivity { face_index: 4 })
    );
}

#[test]
fn closed_bipyramid_is_genus_zero() {
    let input = "\
v 0
v 1
v 2
v 3
v 4
f 0 2 3
f 0 3 4
f 0 4 2
f 1 3 2
f 1 4 3
f 1 2 4
";
    assert_eq!(
        run(input),
        Audit::Ok {
            vertices: 5,
            edges: 9,
            faces: 6,
            euler: 2,
            genus: 0,
        }
    );
}

#[test]
fn torus_is_genus_one() {
    let input = include_str!("../data/torus.mesh");
    assert_eq!(
        run(input),
        Audit::Ok {
            vertices: 15,
            edges: 45,
            faces: 30,
            euler: 0,
            genus: 1,
        }
    );
}

#[test]
fn vertex_ids_need_not_be_dense() {
    // 非连续的大 id 同样按数值排序取见证。
    let input = "\
v 10
v 20
v 30
v 40
f 10 30 20
f 10 20 40
f 10 40 30
f 20 30 40
";
    assert_eq!(
        run(input),
        Audit::Ok {
            vertices: 4,
            edges: 6,
            faces: 4,
            euler: 2,
            genus: 0,
        }
    );
}
