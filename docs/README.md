# xerver: Documentation Index (Rust)

`xerver` is an educational, production-grade systems software project written from scratch in **Rust** on **Linux**.

Rather than relying on modern asynchronous abstractions (such as Tokio, Actix, Hyper, or Pingora), `xerver` is built step-by-step from raw Linux networking primitives (`std::net`, `libc::epoll`, non-blocking I/O, `splice`). The explicit goal is to demystify how the Linux kernel, network stack, and Rust's ownership model collaborate to serve hundreds of thousands of concurrent connections with microsecond-level latency.

---

## Core Philosophy

> **Do not hide behind abstractions until you understand what they are abstracting.**

High-level libraries and frameworks handle connection pooling, non-blocking I/O multiplexing, and backpressure automatically. By implementing these primitives by hand:
1. You learn the cost of kernel-to-user-space transitions.
2. You experience the C10K problem firsthand and discover why thread-per-connection falls over.
3. You implement backpressure and understand how unbounded buffering causes OOM crashes.
4. You benchmark and profile with hardware performance counters, observing cache misses and context switches.
5. You see how Rust's ownership and type system make resource management (`Drop`) and safe concurrency (`Send`/`Sync`) effortless without sacrificing bare-metal performance.

---

## Documentation Navigation

- [**Roadmap & Milestones**](file:///Users/drumilbhati/Documents/Github/xerver/docs/roadmap.md): Detailed 10-stage progression from a single-client TCP echo server to a fully tuned reverse proxy in Rust.
- [**System Architecture**](file:///Users/drumilbhati/Documents/Github/xerver/docs/architecture.md): Event-driven Reactor pattern, dual-socket bridging, connection state machines, and backpressure mechanics.
- [**Tooling & Environment**](file:///Users/drumilbhati/Documents/Github/xerver/docs/tooling-and-environment.md): Setting up the Linux environment, Cargo workflows, and using `epoll`, `strace`, `perf`, `tcpdump`, `ss`, and `wrk`.

---

## Technology Stack

| Domain | Technology |
| :--- | :--- |
| **Language** | Rust (Ownership, borrowing, lifetimes, `Drop`, zero-cost abstractions) |
| **Operating System** | Linux (Debian Bookworm / Kernel 6.x) |
| **Build System** | Cargo (`cargo build --release`, `cargo clippy`, `cargo test`) |
| **I/O Multiplexing** | Linux `epoll` (`libc::epoll_create1`, `libc::epoll_ctl`, `libc::epoll_wait`) |
| **Debugging & Diagnostics** | `gdb`, `lldb`, `strace`, `valgrind` |
| **Profiling & Metrics** | `perf`, `flamegraph`, `ss`, `tcpdump` |
| **Benchmarking** | `wrk`, `hey`, `netcat` |
