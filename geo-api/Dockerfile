FROM rust:1.80 as builder
WORKDIR /app
COPY . .
ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
RUN cargo build --release

FROM debian:bookworm-slim
RUN mkdir -p /app

COPY --from=builder /app/target/release/geo-api /app/
COPY GeoLite2-City.mmdb /app/

WORKDIR /app
EXPOSE 8080
CMD ["/app/geo-api"]