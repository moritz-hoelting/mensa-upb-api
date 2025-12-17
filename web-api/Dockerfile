
FROM rust:latest AS chef
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN OFFLINE=true cargo chef prepare --bin mensa-upb-api --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --bin mensa-upb-api --release --recipe-path recipe.json
COPY . .
RUN OFFLINE=true cargo build --bin mensa-upb-api --release

FROM debian:bookworm-slim AS runtime

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=builder /app/target/release/mensa-upb-api /bin/mensa-upb-api

ENV API_INTERFACE=0.0.0.0

EXPOSE 8080

# What the container should run when it is started.
CMD ["/bin/mensa-upb-api"]