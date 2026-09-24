//! 组合拓扑审计核心。
//!
//! 检查按固定优先级短路：边 → 顶点扇区 → 整体连通。
//! 每步失败都返回按 id 排序最小的见证。

use std::collections::HashMap;

use crate::parse::{index_map, Mesh};

/// 无向边异常类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// 只被一个面使用（边界边）。
    Boundary,
    /// 被两个面以相同方向使用（定向冲突）。
    Orientation,
    /// 被三个或更多面使用（非流形边）。
    NonManifold,
}

impl EdgeKind {
    pub fn label(self) -> &'static str {
        match self {
            EdgeKind::Boundary => "boundary",
            EdgeKind::Orientation => "orientation",
            EdgeKind::NonManifold => "non-manifold",
        }
    }
}

/// 边异常见证：端点为顶点 id，lo < hi。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeAnomaly {
    pub kind: EdgeKind,
    pub lo: u64,
    pub hi: u64,
}

/// 审计发现的拓扑缺陷见证。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// 第一步：最小异常无向边。
    Edge(EdgeAnomaly),
    /// 第二步：最小 id 的非流形顶点（扇区数 != 1，含未被任何面使用）。
    Vertex { id: u64, sectors: usize },
    /// 第三步：不在面 0 所在连通块中的最小面下标（0 起始，按输入顺序）。
    Connectivity { face_index: usize },
}

/// 审计结果：要么通过并给出不变量，要么给出最高优先级的最小见证。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Audit {
    Ok {
        vertices: usize,
        edges: usize,
        faces: usize,
        euler: i64,
        genus: u64,
    },
    Failed(Finding),
}

/// 对解析校验通过的网格执行拓扑审计。
pub fn audit(mesh: &Mesh) -> Audit {
    let idx = index_map(&mesh.vertices);
    let n = mesh.vertices.len();

    // 面解析为内部顶点下标。
    let faces: Vec<[usize; 3]> = mesh
        .faces
        .iter()
        .map(|tri| tri.map(|v| idx[&v]))
        .collect();

    // ---- 第一步：无向边使用计数与方向 ----
    // 每条无向边记录所有使用它的有向边的尾端点；方向相反 <=> 两个尾端点不同。
    let mut edge_tails: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for tri in &faces {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tails.entry(key).or_default().push(a);
        }
    }

    let mut worst_edge: Option<EdgeAnomaly> = None;
    for (&(lo, hi), tails) in &edge_tails {
        let kind = match tails.len() {
            1 => Some(EdgeKind::Boundary),
            2 => {
                // 恰两次使用时，方向必须相反（尾端点相同即同向冲突）。
                if tails[0] == tails[1] {
                    Some(EdgeKind::Orientation)
                } else {
                    None
                }
            }
            _ => Some(EdgeKind::NonManifold),
        };
        if let Some(kind) = kind {
            let anomaly = EdgeAnomaly {
                kind,
                lo: mesh.vertices[lo],
                hi: mesh.vertices[hi],
            };
            let replace = match &worst_edge {
                None => true,
                Some(w) => (anomaly.lo, anomaly.hi) < (w.lo, w.hi),
            };
            if replace {
                worst_edge = Some(anomaly);
            }
        }
    }
    if let Some(anomaly) = worst_edge {
        return Audit::Failed(Finding::Edge(anomaly));
    }

    // ---- 第二步：每个顶点的扇区数 ----
    // 边检查通过后，每条无向边恰有一对反向有向边（dart）。
    // 在半边模型上：alpha 交换对向 dart，sigma 沿面推进，
    // phi = sigma ∘ alpha 的轨道正是顶点扇区；扇区数必须为 1。
    let darts: Vec<(usize, usize)> = faces
        .iter()
        .flat_map(|tri| (0..3).map(move |k| (tri[k], tri[(k + 1) % 3])))
        .collect();
    let mut rev: HashMap<(usize, usize), usize> = HashMap::with_capacity(darts.len());
    for (d, &(a, b)) in darts.iter().enumerate() {
        rev.insert((a, b), d);
    }
    let phi: Vec<usize> = (0..darts.len())
        .map(|d| {
            let (a, b) = darts[d];
            let opposite = rev[&(b, a)];
            // sigma：同一面内下一条 dart，即 (b, 该面下一顶点)。
            (opposite / 3) * 3 + (opposite + 1) % 3
        })
        .collect();

    let mut visited = vec![false; darts.len()];
    let mut sectors = vec![0usize; n];
    for d in 0..darts.len() {
        if visited[d] {
            continue;
        }
        let v = darts[d].0;
        let mut cur = d;
        while !visited[cur] {
            visited[cur] = true;
            cur = phi[cur];
        }
        sectors[v] += 1;
    }

    let mut worst_vertex: Option<(u64, usize)> = None;
    for (i, &s) in sectors.iter().enumerate() {
        if s != 1 {
            let id = mesh.vertices[i];
            let replace = match &worst_vertex {
                None => true,
                Some((w, _)) => id < *w,
            };
            if replace {
                worst_vertex = Some((id, s));
            }
        }
    }
    if let Some((id, s)) = worst_vertex {
        return Audit::Failed(Finding::Vertex { id, sectors: s });
    }

    // ---- 第三步：面经共享边的整体连通性 ----
    let mut parent: Vec<usize> = (0..faces.len()).collect();
    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        let mut r = x;
        while parent[r] != r {
            r = parent[r];
        }
        let mut c = x;
        while parent[c] != r {
            let next = parent[c];
            parent[c] = r;
            c = next;
        }
        r
    }
    // 直接由 dart 表重建“边 -> 两个面”的邻接。
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, tri) in faces.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }
    for fs in edge_faces.values() {
        if fs.len() == 2 {
            let (a, b) = (fs[0], fs[1]);
            let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
            if ra != rb {
                parent[ra] = rb;
            }
        }
    }
    let root0 = find(&mut parent, 0);
    for f in 1..faces.len() {
        if find(&mut parent, f) != root0 {
            return Audit::Failed(Finding::Connectivity { face_index: f });
        }
    }

    // ---- 全部通过：输出不变量 ----
    let v = n as i64;
    let e = edge_tails.len() as i64;
    let f = faces.len() as i64;
    let euler = v - e + f;
    debug_assert!(euler % 2 == 0 && euler <= 2);
    let genus = ((2 - euler) / 2) as u64;
    Audit::Ok {
        vertices: n,
        edges: edge_tails.len(),
        faces: faces.len(),
        euler,
        genus,
    }
}
