FROM rust:1.80 as builder
WORKDIR /app
COPY . .
ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/oss-res /oss-res
ENTRYPOINT ["/oss-res"]