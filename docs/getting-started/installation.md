# Installation

This page covers local prerequisites and repository setup for Windows, Linux, and macOS.

## Required Tooling

Install these on every platform:

- Rust stable with Cargo
- Node.js 20 or newer
- pnpm 10 or newer

Optional:

- PostgreSQL 15 or newer for PostgreSQL-backed deployments
- Tauri CLI for desktop development or packaging

Install the Tauri CLI:

```bash
cargo install tauri-cli
```

## Platform Notes

### Windows

Recommended:

- Rust via `rustup`
- Node.js 20+
- PowerShell 7 or Windows PowerShell
- WebView2 runtime if you intend to use Tauri

### Linux

Recommended:

- Rust via `rustup`
- Node.js 20+
- pnpm enabled via Corepack or installed separately
- desktop WebView dependencies if you intend to use Tauri

### macOS

Recommended:

- Rust via `rustup`
- Node.js 20+
- pnpm enabled via Corepack or installed separately
- Xcode Command Line Tools for native desktop compilation paths

## Clone and Install

Clone the repository:

```bash
git clone https://github.com/Sdkwork-Cloud/sdkwork-api-server.git
cd sdkwork-api-server
```

Install console dependencies:

```bash
pnpm --dir console install
```

Install docs dependencies:

```bash
pnpm --dir docs install
```

## Verify Tooling

Rust:

```bash
rustc --version
cargo --version
```

Node and pnpm:

```bash
node --version
pnpm --version
```

Optional PostgreSQL:

```bash
psql --version
```

## Next Steps

- To run from source, continue with [Source Development](/getting-started/source-development)
- To build release artifacts, continue with [Release Builds](/getting-started/release-builds)
- To preview the docs site locally, run:

```bash
pnpm --dir docs dev
```
