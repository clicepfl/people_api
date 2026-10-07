FROM alpine:3.23 AS runner

RUN apk add openssl

FROM rust:1-alpine3.23 AS build

ARG pkg=people_api

WORKDIR /build
RUN apk add openssl alpine-sdk openssl-dev pkgconfig

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src target/release/$pkg

COPY src src

RUN set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./main

COPY . .

FROM runner

WORKDIR /app

COPY --from=build /build/main ./main
COPY --from=build /build/Rocket.toml .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=1819

RUN chown -R 405 /app
USER 405

EXPOSE 1819

CMD ["/app/main"]

