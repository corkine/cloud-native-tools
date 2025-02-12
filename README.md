# Cloud Natie Tools

这里提供了 Kubernetes 环境运行微服务项目中所诞生的一些工具项目，大部分使用 Rust 开发。

## file-api

提供挂载文件内容读取的 REST 接口的服务，目前主要用于 Calibre 配置挂载、读取和分发。

## geo-api

提供 IP -> 城市 GPS 位置信息的 REST 接口的服务，目前主要用于业务日志记录访问者信息，在 Loki 存储并在 Grafana 地图可视化展示。

## webhook-git-updater

提供 Git 仓库拉取和更新的 REST 接口的服务以及命令行程序，支持 Git 更新调用 Webhook 更新源码（动态语言或 Clojure 热更新环境下服务实时反应更新），也支持作为 init-container 让容器基于最新代码运行（Clojure 生产环境）。

## oss-res

从 Aliyun OSS 下载部署资源到本地，用于初始化容器，常用于配合 CI 系统的 JRE 镜像的可执行 Jar 包下载。