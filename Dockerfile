################################################################################
# Build stage - based on Rust Docker Official image)
# This stage compiles the application.
################################################################################
FROM rust:alpine AS build

WORKDIR /app
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=openapi,target=openapi \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/var/cache/cargo \
    CARGO_HOME=/var/cache/cargo cargo build --locked --release \
    && cp ./target/release/zekurix-server /bin/zekurix-server

################################################################################
# Runtime stage - based on Alpine Docker Official image
# This stage runs the already-compiled binary with minimal dependencies.
################################################################################
FROM alpine:3.24 AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    zekurix \
    && mkdir -p /etc/zekurix
USER zekurix

COPY --from=build /bin/zekurix-server /bin/zekurix-server

EXPOSE 3000

HEALTHCHECK \
    --interval=30s \
    --timeout=5s \
    --start-period=10s \
    --retries=3 \
    CMD wget -Y off --quiet --tries=1 --spider http://127.0.0.1:3000/health || exit 1

CMD ["/bin/zekurix-server"]
