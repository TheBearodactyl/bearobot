FROM rust:1.90-alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    sqlite-dev

WORKDIR /app

COPY . .

RUN mkdir -p db

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache \
    sqlite \
    ca-certificates \
    openssl \
    libgcc

RUN addgroup -g 1001 -S bearobot && \
    adduser -S bearobot -u 1001 -G bearobot

RUN mkdir -p /app/db && \
    chown -R bearobot:bearobot /app

WORKDIR /app

COPY --from=builder /app/target/release/bearobot /app/bearobot
COPY --from=builder /app/db /app/db

RUN chown -R bearobot:bearobot /app

USER bearobot

EXPOSE 2379

CMD ["./bearobot"]
