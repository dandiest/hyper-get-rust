<p align="center">
  <img src="https://img.shields.io/badge/License-MIT-yellow.svg" />
  <img src="https://img.shields.io/badge/Language-Rust-orange.svg" />
  <img src="https://img.shields.io/badge/Status-Stable-brightgreen.svg" /> 
  <img src="https://img.shields.io/badge/Runtime-Tokio-blue.svg" />
  <img src="https://img.shields.io/badge/Library-Reqwest-red.svg" />
</p>

<h1 align="center">🦀 Hyper Get Rust - Multi-threaded Core</h1>

<p align="center">
  A high-performance Command Line Interface (CLI) designed for rapid file acquisition, featuring asynchronous range-based chunking, parallel thread orchestration, and atomic path management.
</p>

---

## 🎓 Educational Disclaimer
This repository is a cornerstone of my **Personal Apprenticeship** in Rust. 
* **Purpose**: This project focuses on mastering high-concurrency patterns and asynchronous network I/O.
* **Evolution**: Building upon basic HTTP requests, Velocity introduces "Distributed Downloading"—partitioning a single resource into segments for simultaneous acquisition.
* **Focus**: Deep dive into **Tokio task spawning**, **Atomic Reference Counting (Arc)** for shared state, and **HTTP Range headers** for partial content delivery.

## 🌟 Features
* **Parallel Orchestration**: Spawns multiple asynchronous tasks to saturate available bandwidth by downloading file segments in parallel.
* **Smart Partitioning**: Automatically calculates byte ranges and offsets to ensure bit-perfect file reconstruction without data overlap.
* **Atomic Path Safety**: Leverages `Arc<String>` to safely share file system paths across multiple thread boundaries without unnecessary allocations.
* **Pre-allocation Logic**: Sets the total file length on disk before writing to prevent file fragmentation and ensure OS-level space reservation.

## 🛠️ Technical Deep Dive
* **Asynchronous I/O**: Utilizes `tokio::fs` and `tokio::io` to perform non-blocking disk operations while the network buffer populates.
* **Thread Synchronization**: Employs `JoinHandle` collection and await logic to ensure the main process remains active until all worker threads report success.
* **Dynamic Range Casting**: Implements safe type conversion between `u64` (file sizes) and `usize` (memory offsets) to maintain 64-bit compatibility.

---

## 🚀 How to Run
1. Clone the repository:
   ```bash
   git clone [https://github.com/dandiest/velocity-download-engine-rust.git](https://github.com/dandiest/velocity-download-engine-rust.git)

2. Build and run:
    ```bash
    cargo run
    
 ---
    
## ⚠️ Troubleshooting & Compatibility

Multi-threaded downloading relies on the server's cooperation. Here is how Hyper Get handles common web hurdles:
Error 403 (Forbidden)

    Root Cause: Server-side Bot Protection.

    Solution: Hyper Get implements professional User-Agent and Accept headers to mimic browser behavior and bypass basic security filters.

SSL/TLS Error

    Root Cause: System Clock Mismatch or invalid certificates.

    Solution: The engine uses native-tls for OS-level validation. Ensure your system clock is synchronized to prevent certificate expiration errors.

Incomplete Message

    Root Cause: Instant connection closure when requesting 0 bytes.

    Solution: We use a Range: bytes=0-1 fallback. This keeps the pipe open long enough to extract metadata without triggering a server-side reset.

Size Not Found

    Root Cause: Missing Ranged Support.

    Solution: If a server doesn't support Range headers, multi-threading is technically impossible. Hyper Get identifies this early to prevent corrupted downloads.

## ⚖️ License & Copyright

Copyright © 2026 *[dandiest]*

This project is licensed under the MIT License.

*You are free to use, study, and modify this code for educational purposes. Professionalism starts with sharing knowledge.*
