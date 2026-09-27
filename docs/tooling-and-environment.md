# Tooling, Debugging & Environment Guide

Because `xerver` leverages Linux-specific kernel interfaces (`epoll`, `splice`, `SO_REUSEPORT`), development on macOS runs inside an Ubuntu Linux container environment (see [platform/LINUX_DEV.md](file:///Users/drumilbhati/Documents/Github/xerver/platform/LINUX_DEV.md)).

---

## 1. Quickstart Development Workflow

### Launching the Linux Container
From the repository root on macOS:

```sh
docker compose -f platform/docker-compose.yml build
docker compose -f platform/docker-compose.yml run --rm --service-ports dev
```

### Compiling with CMake & Ninja
Inside the container:

```sh
# Configure Debug build with symbols
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DCMAKE_CXX_STANDARD=23

# Compile
cmake --build build

# Run tests
ctest --test-dir build --output-on-failure
```

---

## 2. Systems Diagnostics Cheatsheet

### Tracing System Calls (`strace`)
`strace` intercepts and records the system calls made by `xerver`, giving visibility into kernel interactions:

```sh
# Trace network-specific syscalls (socket, bind, accept, epoll_wait, etc.)
strace -f -e trace=network ./build/xerver

# Trace read and write operations including byte counts
strace -f -e trace=read,write,epoll_wait ./build/xerver

# Collect call counts, total time, and errors per syscall
strace -c ./build/xerver
```

### Socket Inspection (`ss` & `netstat`)
Inspect TCP socket buffers, states, and port listeners:

```sh
# List all listening TCP sockets with process names and port numbers
ss -lntp

# Inspect internal TCP state, RTT, and window sizes of active connections
ss -ti

# View socket buffer memory allocations
ss -m
```

### Packet Inspection (`tcpdump`)
Capture and examine network packets flowing through the loopback interface:

```sh
# Monitor TCP handshake and data packets on port 8080
tcpdump -i any -nn port 8080

# Capture raw packets for Wireshark inspection
tcpdump -i any -w /workspace/capture.pcap port 8080
```

---

## 3. Benchmarking & Load Testing

### Using `wrk`
`wrk` is a modern HTTP benchmarking tool capable of generating high load from multiple threads:

```sh
# Benchmark with 4 threads, 100 concurrent connections, for 30 seconds
wrk -t4 -c100 -d30s --latency http://localhost:8080/

# Test Keep-Alive vs connection closure:
# To disable keep-alive and test raw connection handshake overhead:
wrk -t4 -c100 -d30s -H "Connection: close" http://localhost:8080/
```

### Key Metrics to Monitor
- **RPS (Requests Per Second)**: Raw throughput capacity.
- **Latency Distribution**: Mean vs. p99 vs. max latency (identifies tail latency spikes caused by event loop stalls).
- **Socket Queues**: Check whether `Recv-Q` or `Send-Q` in `ss -lnt` are filling up under load.

---

## 4. Profiling with `perf` & FlameGraphs

`perf` samples CPU hardware performance counters to identify execution bottlenecks and cache inefficiencies.

```sh
# Monitor CPU hotspot functions in real time
perf top -p $(pgrep xerver)

# Record CPU cycles with call graphs under load
perf record -F 99 -g -p $(pgrep xerver) -- sleep 20

# Inspect the profile report
perf report -g
```

---

## 5. Memory Safety & Sanitizers

Enable Clang/GCC sanitizers during CMake configuration to catch memory corruption and undefined behavior before runtime crashes:

```sh
# AddressSanitizer & UndefinedBehaviorSanitizer
cmake -S . -B build-asan -G Ninja \
  -DCMAKE_BUILD_TYPE=Debug \
  -DCMAKE_CXX_FLAGS="-fsanitize=address,undefined -fno-omit-frame-pointer"
cmake --build build-asan
```
