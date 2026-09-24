//! 输入解析与格式校验。
//!
//! 文本格式（仅 ASCII）：
//! - 空行与以 `#` 开头的整行注释被忽略；
//! - `v <id>`          声明一个顶点，id 为无符号十进制整数；
//! - `f <a> <b> <c>`   声明一个有向三角面，三个顶点 id 必须互异且已声明。
//!
//! 约束：4..=500 个唯一顶点，4..=2000 个面；未知顶点、重复无向面、
//! 多余或缺失字段、非 ASCII 内容一律拒绝。

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

pub const MIN_VERTICES: usize = 4;
pub const MAX_VERTICES: usize = 500;
pub const MIN_FACES: usize = 4;
pub const MAX_FACES: usize = 2000;

/// 解析后的网格：顶点 id 表（升序、唯一）与有向面列表（顶点 id 三元组）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mesh {
    pub vertices: Vec<u64>,
    pub faces: Vec<[u64; 3]>,
}

/// 输入拒绝原因，携带 1 起始的行号（计数类错误行号为 0）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "line {}: {}", self.line, self.message)
        }
    }
}

impl Error for ParseError {}

fn err(line: usize, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}

/// 解析顶点 id：仅接受无符号十进制，拒绝符号与溢出。
fn parse_id(tok: &str) -> Option<u64> {
    if tok.is_empty() || !tok.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    tok.parse::<u64>().ok()
}

/// 解析完整输入文本，返回网格或首个拒绝原因。
pub fn parse(input: &str) -> Result<Mesh, ParseError> {
    if !input.is_ascii() {
        return Err(err(0, "input must be pure ASCII"));
    }

    let mut vertices: Vec<u64> = Vec::new();
    let mut vertex_set: HashSet<u64> = HashSet::new();
    let mut faces: Vec<[u64; 3]> = Vec::new();
    let mut face_set: HashSet<[u64; 3]> = HashSet::new();

    for (idx, raw) in input.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let toks: Vec<&str> = trimmed.split_whitespace().collect();
        match toks[0] {
            "v" => {
                if toks.len() != 2 {
                    return Err(err(
                        line_no,
                        format!("vertex line expects exactly 2 fields, got {}", toks.len()),
                    ));
                }
                let id = parse_id(toks[1])
                    .ok_or_else(|| err(line_no, format!("invalid vertex id `{}`", toks[1])))?;
                if !vertex_set.insert(id) {
                    return Err(err(line_no, format!("duplicate vertex id {id}")));
                }
                vertices.push(id);
            }
            "f" => {
                if toks.len() != 4 {
                    return Err(err(
                        line_no,
                        format!("face line expects exactly 4 fields, got {}", toks.len()),
                    ));
                }
                let mut tri = [0u64; 3];
                for (k, slot) in tri.iter_mut().enumerate() {
                    *slot = parse_id(toks[k + 1]).ok_or_else(|| {
                        err(line_no, format!("invalid vertex id `{}`", toks[k + 1]))
                    })?;
                }
                if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                    return Err(err(line_no, "face vertices must be pairwise distinct"));
                }
                // 无向面去重：顶点排序后作为键，旋转/翻转均视为重复。
                let mut key = tri;
                key.sort_unstable();
                if !face_set.insert(key) {
                    return Err(err(
                        line_no,
                        format!("duplicate undirected face ({}, {}, {})", tri[0], tri[1], tri[2]),
                    ));
                }
                faces.push(tri);
            }
            other => {
                return Err(err(line_no, format!("unknown directive `{other}`")));
            }
        }
    }

    if vertices.len() < MIN_VERTICES || vertices.len() > MAX_VERTICES {
        return Err(err(
            0,
            format!(
                "vertex count {} out of range {MIN_VERTICES}..={MAX_VERTICES}",
                vertices.len()
            ),
        ));
    }
    if faces.len() < MIN_FACES || faces.len() > MAX_FACES {
        return Err(err(
            0,
            format!(
                "face count {} out of range {MIN_FACES}..={MAX_FACES}",
                faces.len()
            ),
        ));
    }

    // 未知顶点检查（在面数校验之后，保证每条面记录都已被计入）。
    for (i, tri) in faces.iter().enumerate() {
        for &v in tri {
            if !vertex_set.contains(&v) {
                return Err(err(
                    0,
                    format!("face {} references unknown vertex id {v}", i + 1),
                ));
            }
        }
    }

    vertices.sort_unstable();
    Ok(Mesh { vertices, faces })
}

/// 顶点 id -> 内部索引（升序）映射。
pub fn index_map(vertices: &[u64]) -> HashMap<u64, usize> {
    vertices
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect()
}
