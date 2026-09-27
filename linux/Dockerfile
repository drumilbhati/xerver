FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential \
       clang \
       cmake \
       ninja-build \
       gdb \
       strace \
       linux-tools-generic \
       iproute2 \
       iputils-ping \
       net-tools \
       tcpdump \
       netcat-openbsd \
       curl \
       git \
       pkg-config \
       libgtest-dev \
       wrk \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash dev \
    && mkdir -p /workspace \
    && chown -R dev:dev /workspace

WORKDIR /workspace
USER dev

CMD ["bash"]
