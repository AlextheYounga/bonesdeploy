FROM debian:stable

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        bash \
        git \
        rsync \
        sudo \
        openssh-server \
    && rm -rf /var/lib/apt/lists/*
