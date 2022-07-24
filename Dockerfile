FROM rust:1.62-alpine3.16

RUN apk add --no-cache musl-dev
# "search" cargo to force update the local index.
# We do this as a separate step before copying the sources because we want to cache this stage.
# `cargo update` also does the same action, but it requires a cargo project, which we don't have yet.
RUN cargo search --limit 0

RUN mkdir -p /plugin_src
ADD Cargo.toml /plugin_src
ADD src /plugin_src/src
WORKDIR /plugin_src
RUN cargo build --release

# We finished building the plugin, now start building the final image.
FROM alpine:3.16
COPY --from=0 /plugin_src/target/release/docker-logging-plugin /bin/file-logging-plugin
WORKDIR /
RUN mkdir -p /core
ENTRYPOINT ["/bin/file-logging-plugin"]
