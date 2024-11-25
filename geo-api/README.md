# geo-api

这是一个简单 Web 应用程序，可通过 API 获取指定 IP 地址的城市坐标和描述。从 [MaxMind](https://blog.maxmind.com/2019/12/significant-changes-to-accessing-and-using-geolite2-databases/) 下载 GeoLite2-City 数据库到本地根目录的 GeoLite2-City.mmdb 文件。

创建 `.env` 文件或从环境变量提供如下参数，如果不提供则不进行鉴权。

```env
API_USER=admin
API_PASSWORD=admin
```

```bash
cargo run --release
# or use container
docker build -t geo-api:latest .
docker run -it --rm \
    -p 8080:8080 \
    -v GeoLite2-City.mmdb:/app/GeoLite2-City.mmdb \
    -e API_USER=admin \
    -e API_PASSWORD=123 \
    localhost/geo-api:latest
```


## License

MIT
