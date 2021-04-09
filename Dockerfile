FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin serve-log
WORKDIR /home/rust/src/serve-log
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
ADD . ./
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/serve_log*
RUN cargo build --release

FROM alpine:latest
EXPOSE 5000
ENV TZ=Etc/UTC \
    APP_USER=appuser
RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER
RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*
COPY index.txt /usr/src/app/index.txt
COPY --from=builder /home/rust/src/serve-log/target/x86_64-unknown-linux-musl/release/serve-log /usr/src/app/serve-log
RUN chown -R $APP_USER:$APP_USER /usr/src/app
USER $APP_USER
WORKDIR /usr/src/app

CMD ["./serve-log"]