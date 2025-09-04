# Rclone Me

Rclone 的增强版本，允许通过 --config_url 提供 URL，系统启动后会通过它下载配置文件并通过环境变量注入，然后启动 Rclone 本身。或者通过 RCLONE_CONFIG_URL 暴露接口。

参见 [Rclone](https://rclone.org/)。