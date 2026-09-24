# meshcheck — 三角面片组合拓扑审计器

`meshcheck` 审计有向三角面片列表是否构成**组合拓扑意义下的封闭可定向曲面**。
即使每条边都恰好出现两次，网格仍可能在某个顶点处捏成两个互不相连的扇区；
本工具按固定优先级依次检查三类缺陷，并给出最小 id 见证。

> 范围声明：本工具只证明组合拓扑水密（边配对、顶点扇区、整体连通），
> **不检测几何自交**。

## 输入格式

纯 ASCII 文本，逐行解析：

```
# 注释：整行以 # 开头；空行忽略
v 0        # 声明顶点，id 为无符号十进制整数（u64）
v 1
v 2
v 3
f 0 2 1    # 有向三角面：三个顶点 id 必须互异且已声明
f 0 1 3
f 0 3 2
f 1 2 3
```

硬性约束（违反即拒绝，退出码 2）：

- 唯一顶点 4..=500 个，面 4..=2000 个；
- 未知顶点、重复无向面（顶点集合相同，不论旋转/翻转）、多余或缺失字段、
  重复顶点 id、非 ASCII 内容，一律拒绝。

## 审计步骤与见证优先级

1. **边**：每条无向边必须恰由两个面以**相反方向**使用。
   异常类别：`boundary`（仅 1 次使用）、`orientation`（2 次同向）、
   `non-manifold`（≥3 次使用）。见证为按 `(小端点, 大端点)` 字典序最小的异常边。
2. **顶点**：每个顶点的邻接面必须构成**单一扇区**。
   实现：在半边结构上取 `phi = sigma ∘ alpha`，其轨道即顶点扇区；
   扇区数不为 1（含未被任何面使用的顶点）即失败。见证为最小 id 顶点。
3. **整体连通**：全部面经共享边必须属于同一连通块。
   见证为不在面 0 所在连通块中的最小面下标（0 起始，按输入顺序）。

任一步失败即短路返回；全部通过时输出 `V、E、F`、欧拉示性数 `χ = V − E + F`
与整数亏格 `g = (2 − χ) / 2`。

## 输出与退出码

| 情形 | 退出码 | 输出（stdout / stderr） |
| --- | --- | --- |
| 审计通过 | 0 | `OK V=4 E=6 F=4 euler=2 genus=0` |
| 边异常 | 1 | `FAIL edge orientation 0-1 witness=edge(0,1)` |
| 顶点捏合 | 1 | `FAIL vertex non-manifold 0 sectors=2 witness=vertex(0)` |
| 不连通 | 1 | `FAIL connectivity face_index=4 witness=face#4` |
| 输入被拒绝 | 2 | stderr: `REJECT line 5: ...` |

## 使用

```console
$ cargo build --release
$ ./target/release/meshcheck data/tetra.mesh        # 审计文件
OK V=4 E=6 F=4 euler=2 genus=0
$ ./target/release/meshcheck data/torus.mesh        # 亏格 1 环面
OK V=15 E=45 F=30 euler=0 genus=1
$ cat bad.mesh | ./target/release/meshcheck -       # 标准输入
FAIL edge boundary 0-2 witness=edge(0,2)
```

## 测试

```console
$ cargo test
```

覆盖：翻面（定向冲突）、边界边、非流形边、共享单顶点的双壳（顶点捏合）、
互不相交的双壳（整体连通）、合法封闭体（四面体、三角双锥、亏格 1 环面），
以及未知顶点、重复无向面、额外字段、退化面、重复顶点、数量越界、非 ASCII
等拒绝路径。

## Docker / Compose

```console
$ docker compose run --rm meshcheck                          # 审计内置示例
$ docker compose run --rm meshcheck data/torus.mesh          # 审计指定文件
$ cp your.mesh data/ && docker compose run --rm meshcheck data/your.mesh
$ cat your.mesh | docker compose run --rm -T meshcheck -     # 标准输入
$ docker compose --profile test run --rm meshcheck-test      # 容器内 cargo test
```

## 项目结构

```
meshcheck/
├── Cargo.toml
├── Dockerfile            # 多阶段：rust 构建 -> debian-slim 运行
├── compose.yaml          # meshcheck 服务 + meshcheck-test（profile: test）
├── data/                 # 示例：tetra.mesh（g=0）、torus.mesh（g=1）
├── src/
│   ├── lib.rs
│   ├── parse.rs          # 格式解析与硬性校验
│   ├── audit.rs          # 边 -> 顶点扇区 -> 连通性 三级审计
│   └── main.rs           # CLI：退出码 0/1/2
└── tests/
    ├── audit.rs          # 拓扑场景：翻面/边界/捏合/不连通/封闭体
    └── cli.rs            # 端到端：退出码与输出格式、拒绝路径
```
