FROM debian:bookworm

ENV DEBIAN_FRONTEND=noninteractive

# Update and install complete systems development toolchain, full man pages, and editors
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Compilers & Build Tools
    build-essential \
    clang \
    clang-format \
    clang-tidy \
    cmake \
    ninja-build \
    pkg-config \
    libgtest-dev \
    ca-certificates \
    # Debugging & Systems Profiling
    gdb \
    strace \
    valgrind \
    # Full Linux Manual Pages (Syscalls, Sockets, Standards)
    man-db \
    manpages \
    manpages-dev \
    # Text Editors
    neovim \
    vim \
    nano \
    # Terminal Ergonomics & Utilities
    tmux \
    zsh \
    git \
    ripgrep \
    fd-find \
    tree \
    htop \
    less \
    curl \
    wget \
    jq \
    sudo \
    # Networking Stack & Benchmarking
    iproute2 \
    iputils-ping \
    net-tools \
    tcpdump \
    netcat-openbsd \
    dnsutils \
    socat \
    wrk \
    && mandb -c \
    && rm -rf /var/lib/apt/lists/*

# Create dev user with passwordless sudo
RUN useradd --create-home --shell /bin/bash dev \
    && echo "dev ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && mkdir -p /workspace \
    && chown -R dev:dev /workspace

WORKDIR /workspace
USER dev

CMD ["bash"]
