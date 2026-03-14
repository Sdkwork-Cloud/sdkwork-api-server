# 健康检查与 Metrics

各服务都暴露健康检查和 metrics 端点。

## 健康检查地址

- gateway: `http://127.0.0.1:8080/health`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`

## Metrics 地址

- gateway: `http://127.0.0.1:8080/metrics`
- admin: `http://127.0.0.1:8081/metrics`
- portal: `http://127.0.0.1:8082/metrics`

## 示例

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8081/admin/health
curl http://127.0.0.1:8082/portal/health
```

```bash
curl http://127.0.0.1:8080/metrics
curl http://127.0.0.1:8081/metrics
curl http://127.0.0.1:8082/metrics
```
