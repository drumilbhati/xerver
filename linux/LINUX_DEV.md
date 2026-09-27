# Linux development environment on macOS

This repository uses Docker Desktop's Linux VM. The source directory is mounted
at `/workspace` inside an Ubuntu 24.04 container, so edits made on macOS are
immediately visible in Linux and build artifacts stay in the repository.

## Start Docker Desktop

Install Docker Desktop for Mac and start it. Verify the daemon is reachable:

```sh
docker info
```

## Build and enter the environment

Run these commands from the repository root:

```sh
docker compose -f linux/docker-compose.yml build
docker compose -f linux/docker-compose.yml run --rm --service-ports dev
```

The shell is now running on Linux. Check it with:

```sh
uname -a
cat /etc/os-release
```

The container includes C++23 tooling, CMake, Ninja, gdb, strace, perf support,
tcpdump, `ss`, netcat, curl, and wrk.

## Verify epoll

Linux headers and the `epoll` system call are available inside the container:

```sh
cat >/tmp/epoll_check.cpp <<'EOF'
#include <sys/epoll.h>
#include <unistd.h>
#include <iostream>

int main() {
    int fd = epoll_create1(EPOLL_CLOEXEC);
    if (fd == -1) return 1;
    std::cout << "epoll is available\n";
    close(fd);
}
EOF
g++ -std=c++20 -Wall -Wextra -pedantic /tmp/epoll_check.cpp -o /tmp/epoll_check
/tmp/epoll_check
```

## Typical workflow

Inside the container, configure and build with:

```sh
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DCMAKE_CXX_STANDARD=23
cmake --build build
```

Run a server, then from another macOS terminal test a published port:

```sh
curl http://localhost:8080/
nc -vz localhost 8080
```

For system-call tracing:

```sh
strace -f -e trace=network ./build/xerver
```

For socket inspection:

```sh
ss -lntp
```

`perf` may require the capabilities configured in `docker-compose.yml` and
Docker Desktop's current kernel settings. `epoll`, sockets, gdb, strace, and
the compiler work without those extra permissions.

## Important boundary

The application runs inside Linux, which is exactly what is needed for
`epoll`. Docker Desktop itself runs a lightweight Linux VM under macOS. This
is different from compiling on macOS: macOS has `kqueue`, while Linux provides
`epoll`.
