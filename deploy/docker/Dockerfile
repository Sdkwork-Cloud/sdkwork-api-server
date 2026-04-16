FROM debian:bookworm-slim

ENV SDKWORK_HOME=/opt/sdkwork \
    SDKWORK_CONFIG_DIR=/opt/sdkwork/config \
    SDKWORK_BOOTSTRAP_PROFILE=prod \
    SDKWORK_BOOTSTRAP_DATA_DIR=/opt/sdkwork/data \
    SDKWORK_ADMIN_SITE_DIR=/opt/sdkwork/sites/admin/dist \
    SDKWORK_PORTAL_SITE_DIR=/opt/sdkwork/sites/portal/dist \
    SDKWORK_WEB_BIND=0.0.0.0:3001 \
    SDKWORK_GATEWAY_BIND=127.0.0.1:8080 \
    SDKWORK_ADMIN_BIND=127.0.0.1:8081 \
    SDKWORK_PORTAL_BIND=127.0.0.1:8082

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 sdkwork \
    && useradd --system --uid 10001 --gid 10001 --home-dir /var/lib/sdkwork --create-home sdkwork

WORKDIR /opt/sdkwork

COPY bin/ ./bin/
COPY sites/ ./sites/
COPY data/ ./data/

RUN mkdir -p config var/data var/log var/run \
    && chown -R sdkwork:sdkwork /opt/sdkwork /var/lib/sdkwork \
    && chmod 755 /opt/sdkwork/bin/router-product-service

USER sdkwork

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=5 \
  CMD curl --fail --silent http://127.0.0.1:3001/api/v1/health || exit 1

ENTRYPOINT ["./bin/router-product-service"]
