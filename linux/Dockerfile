FROM debian:bookworm

ENV DEBIAN_FRONTEND=noninteractive

# 1. Install development toolchain, full man pages, and networking utilities
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Compilers & Build Tools
    build-essential \
    clang \
    clang-format \
    clang-tidy \
    pkg-config \
    ca-certificates \
    # Debugging & Systems Profiling
    gdb \
    strace \
    valgrind \
    # Full Linux Manual Pages
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

# 2. Create dev user with passwordless sudo
RUN useradd --create-home --shell /bin/bash dev \
    && echo "dev ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && mkdir -p /workspace \
    && chown -R dev:dev /workspace

# 3. Permanently install modern Rust & Cargo via rustup for dev user
USER dev
ENV RUSTUP_HOME=/home/dev/.rustup \
    CARGO_HOME=/home/dev/.cargo \
    PATH=/home/dev/.cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

WORKDIR /workspace

CMD ["bash"]
