# -----------------------------
# Chef base
# -----------------------------
FROM rust:alpine AS chef
# SQLx offline mode
ENV SQLX_OFFLINE true

# Alpine build dependencies
RUN apk add --no-cache curl bash musl-dev openssl-dev pkgconfig

# Install cargo-chef
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef -y

WORKDIR /app

# -----------------------------
# Planner
# -----------------------------
FROM chef AS planner
COPY . .
RUN OFFLINE=true cargo chef prepare --recipe-path recipe.json

# -----------------------------
# Builder
# -----------------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN OFFLINE=true cargo build --release \
    --bin mensa-upb-api \
    --bin mensa-upb-scraper \
    --bin scraper-cli

# =====================================================
# Runtime image: scraper (cron-based)
# =====================================================
FROM alpine:latest AS scraper-runtime
WORKDIR /app

RUN apk add --no-cache ca-certificates tzdata dcron tini

RUN echo "0 0/8 * * * /app/mensa-upb-scraper >> /var/log/cron.log 2>&1" \
    > /etc/crontabs/root && \
    touch /var/log/cron.log

COPY --from=builder /app/target/release/mensa-upb-scraper /app/mensa-upb-scraper
COPY --from=builder /app/target/release/scraper-cli /app/scraper-cli

ENTRYPOINT ["/sbin/tini", "--"]
CMD sh -c 'env > /etc/environment && crond -l 2 && tail -f /var/log/cron.log'

# =====================================================
# Runtime image: API
# =====================================================
FROM alpine:latest AS api-runtime

ARG UID=10001
RUN adduser -D -H -u "${UID}" appuser

USER appuser

COPY --from=builder /app/target/release/mensa-upb-api /bin/mensa-upb-api

ENV API_INTERFACE=0.0.0.0
EXPOSE 8080

CMD ["/bin/mensa-upb-api"]