# Module 6: Concurrency Web Server

This repository contains the Advanced Programming Module 6 tutorial project that builds a simple Rust web server step by step, starting from a single-threaded TCP listener and ending with a multithreaded server backed by a thread pool.

## How to run

1. Run `cargo build` to compile the project.
2. Run `cargo run` from the repository root.
3. Open `http://127.0.0.1:7878` in a browser or send a request with a tool such as `curl`.
4. Stop the server with `Ctrl+C` before running the next milestone version.

## Commit 1 Reflection notes

In this milestone I changed the program from a trivial binary into a TCP listener that binds to `127.0.0.1:7878` and continuously accepts incoming browser connections. The `incoming()` iterator yields `Result<TcpStream, Error>` values, so the code unwraps each stream before passing the successful connection into `handle_connection`. Inside `handle_connection`, `BufReader` wraps the mutable stream reference so the program can read request data line by line instead of dealing with raw byte buffers manually. The iterator chain on `lines()` converts each incoming line into a `String`, unwraps each I/O result, stops at the first empty line, and collects the HTTP request headers into a `Vec<String>` for inspection. That stopping condition matters because an empty line marks the end of the HTTP request headers, so the server avoids waiting forever for more input when it only needs the request head. Printing the collected request makes the browser-server interaction visible and helps explain what the server will need to parse in the next milestones. This step also made the single-threaded nature of the server concrete, because every accepted connection is handled immediately in the main loop and there is no parallel work yet.
