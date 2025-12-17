FROM rust:latest AS chef
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN OFFLINE=true cargo chef prepare --bin mensa-upb-scraper --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --bin mensa-upb-scraper --release --recipe-path recipe.json
COPY . .
RUN OFFLINE=true cargo build --bin mensa-upb-scraper --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y ca-certificates cron

RUN echo "0 0 * * * /app/mensa-upb-scraper >> /var/log/cron.log 2>&1" > /etc/cron.d/mensa_upb_scraper
RUN chmod 0644 /etc/cron.d/mensa_upb_scraper
RUN crontab /etc/cron.d/mensa_upb_scraper
RUN touch /var/log/cron.log

COPY --from=builder /app/target/release/mensa-upb-scraper /app/mensa-upb-scraper

CMD env > /etc/environment && cron && tail -f /var/log/cron.log