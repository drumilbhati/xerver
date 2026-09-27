# Rust Ownership & Memory Model Primer
### *A Fast Systems Engineering Guide for C++ / Java Developers*

---

## 1. The Big Picture: Why Ownership Exists

Every programming language must solve one fundamental question: **When and how do we free memory and close OS resources?**

| Language Paradigm | Mechanism | Tradeoff |
| :--- | :--- | :--- |
| **C / C++** | Manual (`malloc`/`free`, `close`) or RAII | Prone to bugs: memory leaks, use-after-free, double-free crashes. |
| **Java / Go / Python** | Garbage Collector (GC) | Background stop-the-world pauses, memory bloat, high tail latency. |
| **Rust** | **Compile-Time Ownership Tracking** | **Zero runtime overhead, no GC, no memory leaks, no dangling pointers.** |

In Rust, the compiler tracks the lifetime of every piece of data. When an owner goes out of scope, the compiler automatically inserts cleanup code (`drop()`).

---

## 2. Rule 1: Every Value Has Exactly One Owner

Think of a value in memory like a physical, unique key.

### In C++ / Java (Assignment copies or duplicates pointers):
```cpp
// C++: Deep copy (allocates new heap memory)
std::string a = "GET / HTTP/1.1";
std::string b = a; 
// Both 'a' and 'b' exist and point to different or shared heap buffers.
```

### In Rust (Assignment is a MOVE by default):
```rust
let a = String::from("GET / HTTP/1.1");
let b = a; // Ownership of the heap buffer is MOVED from 'a' to 'b'!

// println!("{}", a); 
// ^ COMPILER ERROR: Value borrowed here after move!
```

```text
Step 1: let a = String::from(...)
[Variable a on Stack] ---> [Heap: "GET / HTTP/1.1"]

Step 2: let b = a;
[Variable b on Stack] ---> [Heap: "GET / HTTP/1.1"]
[Variable a (DEAD)]   -x-> No longer accessible!
```

### Why this rule exists:
When a variable leaves its `{ }` scope, Rust calls `drop()` to free its memory.
If both `a` and `b` were allowed to own the same heap buffer, both would attempt to free it when they exit scope, causing a **Double-Free Crash**. By invalidating `a`, only `b` frees the memory.

---

## 3. Rule 2: Borrowing (References)

You do not always want to give away full ownership of your data. Often, you just want another function to read it or mutate it temporarily. This is called **Borrowing**.

References come in two distinct flavors, following the **Readers-Writer Lock** principle:

```text
             ┌─────────────────────────────┐
             │ What reference do you need? │
             └──────────────┬──────────────┘
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
  Immutable Borrow (&T)           Mutable Borrow (&mut T)
  - Read-only                     - Read and Write
  - Unlimited readers allowed     - ONLY ONE writer allowed
  - Nobody can modify data        - NO other readers or writers allowed
```

### The Golden Law of Rust:
> **You can have ANY number of immutable references (`&T`), OR exactly ONE mutable reference (`&mut T`), but NEVER BOTH at the same time.**

### Why this rule prevents real bugs:
In C++, modifying a container while iterating over it causes memory corruption (Iterator Invalidation):

```cpp
// C++: Undefined Behavior / Silent Crash!
std::vector<int> v = {1, 2, 3};
for (auto& x : v) {
    v.push_back(4); // Reallocates heap memory, invalidating pointer 'x'!
}
```

In Rust, the compiler rejects this immediately at compile time:

```rust
let mut v = vec![1, 2, 3];
for x in &v {        // Borrowed as IMMUTABLE (&v)
    v.push(4);      // COMPILER ERROR: Cannot borrow `v` as mutable while borrowed as immutable!
}
```

---

## 4. Rule 3: Slices (`&[u8]` and `&str`) — The Core of Zero-Copy

A **slice** is a view into contiguous memory. It does not own the memory; it is simply a fat pointer consisting of two 64-bit numbers on the stack:
1. `ptr`: Memory address of the first byte.
2. `len`: Number of elements.

```rust
let buffer: [u8; 1024] = [0; 1024]; // 1024-byte array on the stack

// Suppose we read 15 bytes from a network socket:
let request_bytes: &[u8] = &buffer[0..15]; // Pointer to start + length 15
```

```text
[ buffer on Stack: 1024 bytes ]
['G']['E']['T'][' ']['/']['a']['p']['i'] ... [0][0][0]
  ▲
  │
[ request_bytes Slice on Stack: 16 bytes ]
  ptr: points to buffer[0]
  len: 8
```

### Why this matters for high-performance servers:
When parsing an HTTP request:
- Instead of creating new heap `String` allocations for the HTTP Method (`"GET"`), Path (`"/api"`), and Headers...
- You simply slice the original read buffer as `&str`.
- **Zero heap allocations (`malloc`), zero copies, maximum performance.**

---

## 5. How Ownership Maps to our Reverse Proxy (`xerver`)

| Server Component | Rust Ownership Concept | What it Guarantees |
| :--- | :--- | :--- |
| `TcpListener::bind(...)` | Returns an **owned** listener | Socket descriptor opens; closes automatically on `drop`. |
| `listener.accept()` | Returns an **owned** `TcpStream` | Represents a client connection. Can be safely moved to worker threads. |
| `stream.read(&mut buf)` | Requires an exclusive **`&mut [u8]`** | Guarantees nobody reads or overwrites the buffer while the kernel writes to it. |
| HTTP Request Parsing | Borrows byte slices **`&[u8]` / `&str`** | Zero-copy header inspection directly over the network buffer. |
| Client Disconnect | Socket goes out of scope $\rightarrow$ `drop()` | File descriptor is closed immediately by the OS. Zero leaks. |

---

## 6. Essential Rust Syntax Cheatsheet for Networking

### 1. Error Handling (`Result<T, E>` and `?`)
Rust has no exceptions. Functions that can fail return `Result`:
```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```
The **`?` operator** unwraps success, or early-returns the error:
```rust
// If bind() succeeds, listener gets the TcpListener.
// If bind() fails, the function returns Err immediately!
let listener = TcpListener::bind("0.0.0.0:8080")?;
```

### 2. Pattern Matching (`match`)
```rust
match stream.read(&mut buffer) {
    Ok(0) => println!("Client disconnected cleanly"),
    Ok(n) => println!("Read {} bytes from client", n),
    Err(e) => eprintln!("Read error: {}", e),
}
```
