# file-api

这是一个简单 Web 应用程序，可通过 API 获取指定目录文件内容。

创建 `.env` 文件或从环境变量提供：

```env
API_USER=admin
API_PASSWORD=admin
```

```bash
cargo run --release
# or use container
docker build -t file-api:latest .
docker run -it --rm \
    -p 8080:8080 \
    -v ../config:/app/config \
    -e API_USER=admin \
    -e API_PASSWORD=123 \
    localhost/file-api:latest
```

## Why this?

在 Kubernetes 环境中可能需要将 Pod 挂载的各种 Volume 目录数据进行备份，我使用 OneDrive Business，它可以使用 rClone 进行数据读写，但需要 OAuth 认证且时间很短。在非云原生环境中，我通过在每个节点执行 `rclone mount` 来维持认证，通过 cronjob 实现定时数据备份。但在云原生的高度分布式环境中，创建 rclone Pod 后，认证信息很难和 Pod 的生命周期保持一致，通过 ConfigMap 或 Secret 读写 rclone 配置然后让 CronJob CR 共享凭证以执行备份不符合 ConfigMap 和 Secret 不可变的最佳实践。

本应用旨在提供一种通用的解决办法的同时解决此问题，即暴露一个非常轻量（3MB内存）的服务，通过 Basic Auth 认证提供目标文件内容读取，此服务可挂载共享 rClone 配置目录，rClone 通过其他机制实现配置文件的认证更新，然后由此服务通过 REST 接口授权提供认证信息。

在我的用例中，我创建了一个 ConfigMap 提供本应用所需的 AUTH_USER 和 AUTH_PASSWORD，然后暴露为此应用的环境变量，让此服务提供 rClone 配置文件读取能力。之后，CronJob CR 挂载相同的此 ConfigMap，调用 pg_dump 等工具在 init-container 实现数据备份，通过 wget 通过此服务读取 rclone 配置，之后执行 rclone sync 实现数据向远端的同步。这种模式实现了一种任意数据（可能来自于 hostPath，initContainer 的临时数据）通过 CronJob 备份的解决方案，同时提供了分布式环境下 rClone 认证信息安全分发的能力。


## License

MIT
