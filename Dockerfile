# Using the `rust-musl-builder` as base image, instead of
# the official Rust toolchain
FROM clux/muslrust:nightly AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo +nightly chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo +nightly chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin serve-log

FROM alpine AS runtime
ENV RUST_BACKTRACE=1
EXPOSE 5000
ENV TZ=Etc/UTC \
    APP_USER=appuser
RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER
RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*
COPY index.txt /usr/src/app/index.txt
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/serve-log /app/serve-log
RUN chown -R $APP_USER:$APP_USER /usr/src/app
USER $APP_USER
WORKDIR /app
CMD ["/app/serve-log"]
