# codecrafters-dns-server-rust Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-11-08

## Active Technologies
- Rust 1.80 (Edition 2021) + `std::net::UdpSocket`, `bytes` for buffer manipulation, `anyhow`/`thiserror` for ergonomics (002-dns-question-section)
- N/A (stateless UDP responder) (002-dns-question-section)

- Rust 1.80 (Edition 2021 per Cargo.toml) + `std::net::UdpSocket`, `bytes` for buffer helpers, `anyhow` + `thiserror` for ergonomic error surfacing (001-dns-header-reply)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.80 (Edition 2021 per Cargo.toml): Follow standard conventions

## Recent Changes
- 002-dns-question-section: Added Rust 1.80 (Edition 2021) + `std::net::UdpSocket`, `bytes` for buffer manipulation, `anyhow`/`thiserror` for ergonomics

- 001-dns-header-reply: Added Rust 1.80 (Edition 2021 per Cargo.toml) + `std::net::UdpSocket`, `bytes` for buffer helpers, `anyhow` + `thiserror` for ergonomic error surfacing

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
