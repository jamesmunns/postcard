# Persistent container that rebuilds the postcard specs when main changes.
# Build with tools/ as the context, so the install scripts can be copied in:
#
# ```sh
# docker build -f tools/deploybot/Dockerfile tools
# ```
FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl git xz-utils \
  && rm -rf /var/lib/apt/lists/*

COPY install-mdbook.sh install-tracey.sh /tmp/install/
RUN /tmp/install/install-mdbook.sh /usr/local/bin \
  && /tmp/install/install-tracey.sh /usr/local/bin \
  && rm -rf /tmp/install

COPY deploybot/deploy.sh deploybot/deploybot.sh /usr/local/bin/

# HOME holds tracey's daemon state. WORK_DIR is world-writable so the
# container can run as any user.
ENV REPO_URL=https://github.com/jamesmunns/postcard.git \
  BRANCH=main \
  OUT_DIR=/srv/deploybot \
  WORK_DIR=/var/lib/deploybot \
  HOME=/var/lib/deploybot \
  INTERVAL=300
RUN mkdir -p /var/lib/deploybot && chmod 1777 /var/lib/deploybot

CMD ["/usr/local/bin/deploybot.sh"]
