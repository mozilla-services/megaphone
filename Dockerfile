# Docker 17.05 or higher required for multi-stage builds
FROM rustlang/rust:nightly as builder

ADD . /app
WORKDIR /app

RUN \
    apt-get -qq update && \
    apt-get -qq install -y default-libmysqlclient-dev && \
    \
    cargo --version && \
    rustc --version && \
    cargo install --root /app


FROM debian:stretch-slim

MAINTAINER <pjenvey@underboss.org>

RUN \
    groupadd --gid 10001 app && \
    useradd --uid 10001 --gid 10001 --home /app --create-home app && \
    \
    apt-get -qq update && \
    apt-get -qq install -y default-libmysqlclient-dev && \
    rm -rf /var/lib/apt/lists

COPY --from=builder /app/bin /app/bin

WORKDIR /app
USER app

CMD ["/app/bin/megaphone"]
