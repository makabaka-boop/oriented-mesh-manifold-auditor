//! meshcheck — 三角面片模型的组合拓扑审计器。
//!
//! 审计分三步，按固定优先级报告最小 id 见证：
//! 1. 每条无向边必须恰由两个面以相反方向使用；
//! 2. 每个顶点的面片邻接必须构成单一扇区（排除顶点捏合）；
//! 3. 全部面经共享边必须整体连通。
//!
//! 只证明组合拓扑水密，不检测几何自交。

pub mod audit;
pub mod parse;

pub use audit::{audit, Audit, EdgeAnomaly, EdgeKind, Finding};
pub use parse::{parse, Mesh, ParseError};
