# System Architecture (Rust)

This document describes the architectural design of `xerver` in Rust, including the event-driven Reactor pattern, connection lifecycle, buffer management, and backpressure mechanisms.

---

## 1. High-Level System Topology

```mermaid
flowchart TB
    subgraph ClientLayer ["Client Layer"]
        C1["Client 1"]
        C2["Client 2"]
        C3["Client N"]
    end

    subgraph CoreEngine ["xerver Core Engine"]
        direction TB
        subgraph NetLoop ["Event Multiplexing (Reactor)"]
            EL["EventLoop (epoll_wait)"]
            CD["Channel / Token Dispatcher"]
            EL --> CD
        end

        subgraph Pipeline ["HTTP & Proxy Pipeline"]
            RL["Rate Limiter (Atomic Token Bucket)"]
            HP["HTTP Parser (Zero-Copy &[u8])"]
            CC["Response Cache (Arc<RwLock<LRU>>)"]
            LB["Load Balancer (Least-Conn / RR)"]
            BP["Backpressure Controller"]
            HP --> RL --> CC --> LB --> BP
        end

        subgraph UpstreamMgmt ["Upstream Connection Manager"]
            CP["Connection Pool"]
            HC["Health Checker"]
            CP <--> HC
        end

        CD <--> Pipeline
        Pipeline <--> UpstreamMgmt
    end

    subgraph UpstreamCluster ["Upstream Backends"]
        U1["Upstream 1 (:8081)"]
        U2["Upstream 2 (:8082)"]
        U3["Upstream 3 (:8083)"]
    end

    ClientLayer <==>|"Client TCP Sockets"| NetLoop
    UpstreamMgmt <==>|"Pooled Upstream TCP Sockets"| UpstreamCluster
```

---

## 2. Event-Driven Reactor Pattern

`xerver` uses an asynchronous, non-blocking **Reactor Pattern**:
- A single thread monitors thousands of file descriptors via `libc::epoll_wait`.
- When an I/O event occurs, the kernel wakes `epoll_wait`, returning an array of triggered `libc::epoll_event` structs.
- The event loop dispatches each event to its corresponding socket token (`handle_read()`, `handle_write()`, `handle_close()`).

### Scaled Model: Thread-Per-Core (Shared-Nothing)
To scale across multiple CPU cores without lock contention, `xerver` can be deployed in a **Thread-Per-Core** architecture:
- Each worker thread runs its own isolated `EventLoop` and `epoll` instance.
- Kernel load-balances incoming connections using the Linux socket flag `SO_REUSEPORT`.
- Rust's type system statically verifies thread boundaries via the `Send` and `Sync` traits.

---

## 3. Transaction Lifecycle (Dual-Socket Bridge)

A reverse proxy request coordinates two separate TCP connections: **Downstream (Client)** and **Upstream (Backend)**.

```mermaid
sequenceDiagram
    autonumber
    participant C as Client
    participant P as xerver (Proxy)
    participant U as Upstream Backend

    C->>P: TCP 3-Way Handshake (SYN, SYN-ACK, ACK)
    Note over P: epoll notifies listen_fd (EPOLLIN)
    P->>P: accept() non-blocking -> client TcpStream
    P->>P: Register client fd in epoll for EPOLLIN

    C->>P: HTTP Request (GET /api/data)
    Note over P: epoll notifies client fd (EPOLLIN)
    P->>P: Read bytes into Client Buffer
    P->>P: Parse HTTP Request (Zero-copy &[u8] slice)
    P->>P: Check Cache & Rate Limiting

    alt Cache Hit
        P->>C: Write Cached Response
    else Cache Miss
        P->>P: Select Upstream via Load Balancer
        P->>U: Acquire or connect() non-blocking upstream socket
        P->>U: Forward HTTP Request
        Note over P,U: Upstream processes request
        U->>P: HTTP Response Stream
        Note over P: epoll notifies upstream fd (EPOLLIN)
        P->>P: Stream upstream bytes to client write buffer
        P->>C: Flush bytes to client (EPOLLOUT)
        P->>P: Return upstream socket to Connection Pool
    end
```

---

## 4. Backpressure & Flow Control

Handling **speed mismatches**:
- **Fast Upstream** (1 Gbps local LAN backend)
- **Slow Client** (50 KBps mobile network)

If the proxy blindly reads from the upstream as fast as possible without checking if the client can accept data, user-space memory will grow indefinitely, exhausting system RAM and triggering the Linux **OOM Killer**.

### The Solution: High/Low Watermarks

```mermaid
stateDiagram-v2
    [*] --> NormalFlow

    NormalFlow --> BackpressureActive: Outbound client buffer > High-Water Mark (e.g. 64KB)
    note right of BackpressureActive
        1. Remove EPOLLIN from Upstream FD
        2. Kernel pauses reading from upstream
        3. Upstream TCP receive window fills up
        4. Upstream backend automatically throttles
    end note

    BackpressureActive --> NormalFlow: Outbound client buffer < Low-Water Mark (e.g. 16KB)
    note right of NormalFlow
        1. Client caught up reading data
        2. Re-enable EPOLLIN on Upstream FD
        3. Resume streaming upstream response
    end note
```

---

## 5. Memory Management & Zero-Copy in Rust

### Zero-Allocation Hot Path
1. **Borrowing & Slices**: HTTP parsing borrows slices of the raw byte buffer (`&[u8]`, `&str`) without allocating dynamic `String` instances on the heap.
2. **Buffer Pools**: Pre-allocated byte vectors (`Vec<u8>`) are recycled using an object pool to avoid memory allocator thrashing.
3. **Linux `splice(2)` Zero-Copy Relay**:
   Piping data directly between two socket buffers inside the Linux kernel:
   $$\text{Upstream Socket Buffer} \xrightarrow{\text{splice}} \text{Pipe Buffer} \xrightarrow{\text{splice}} \text{Client Socket Buffer}$$
