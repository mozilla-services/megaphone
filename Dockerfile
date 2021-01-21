# Docker 17.05 or higher required for multi-stage builds
# NOTE: this builds w/ a nightly version (specified in rust-toolchain)
FROM rust:1.45-buster as builder

ADD . /app
WORKDIR /app

RUN \
    apt-get -qq update && \
    apt-get -qq install -y default-libmysqlclient-dev libssl-dev && \
    \
    cargo --version && \
    rustc --version && \
    cargo install --path . --locked --root /app


FROM debian:buster-slim

RUN \
    groupadd --gid 10001 app && \
    useradd --uid 10001 --gid 10001 --home /app --create-home app && \
    \
    apt-get -qq update && \
    apt-get -qq install -y default-libmysqlclient-dev libssl-dev libcurl4 && \
    rm -rf /var/lib/apt/lists

COPY --from=builder /app/bin /app/bin
COPY --from=builder /app/version.json /app

WORKDIR /app
USER app

# override rocket's dev env defaulting to localhost
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_LOG off

CMD ["/app/bin/megaphone"]
