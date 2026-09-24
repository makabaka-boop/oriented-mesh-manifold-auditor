# 构建阶段：编译 release 二进制
FROM rust:1.82-slim AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY tests ./tests
COPY data ./data
RUN cargo build --release

# 运行阶段：仅携带二进制与示例数据
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/meshcheck /usr/local/bin/meshcheck
COPY data ./data
ENTRYPOINT ["meshcheck"]
CMD ["data/tetra.mesh"]
