# Health and Metrics

The services expose both health and metrics endpoints.

## Health Endpoints

- gateway: `http://127.0.0.1:8080/health`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`

## Metrics Endpoints

- gateway: `http://127.0.0.1:8080/metrics`
- admin: `http://127.0.0.1:8081/metrics`
- portal: `http://127.0.0.1:8082/metrics`

## Example Checks

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

## Operational Expectations

Use health endpoints for:

- liveness checks
- startup validation
- smoke tests

Use metrics endpoints for:

- Prometheus scraping
- request-rate monitoring
- latency tracking
- service-level troubleshooting
