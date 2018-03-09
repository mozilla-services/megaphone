FROM scratch

MAINTAINER <pjenvey@underboss.org>

ADD target/x86_64-unknown-linux-musl/release/megaphone /app
EXPOSE 80

CMD ["/app/megaphone"]
