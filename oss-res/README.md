# oss-res

一个 Rust CLI 程序，用于从 Aliyun OSS 下载二进制文件（或压缩包并解压）并作为初始化容器为主镜像提供资源，常用于实现比如对 JRE 镜像的 jar 包动态运行。

## Develop

```bash
cargo run -- --oss-config=eyJvc3NfYnVja2V0IjoUIn0= --file=/projectA/deploy.zip --output=temp --unzip
```

## Build

```bash
docker build -t corkine/oss-res:0.0.1 .
docker run --rm -i -v .:/app/temp corkine/oss-res:0.0.1 --oss-config=eyJvc3NfYnVja2V0IjoUIn0= --file=/projectA/deploy.zip --output=/app/temp --unzip
```