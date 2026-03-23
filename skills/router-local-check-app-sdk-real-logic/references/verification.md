# Router Local Check Verification

Run the narrowest useful set first, then broaden before completion:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd console
pnpm install
pnpm typecheck
pnpm build
pnpm tauri:build
```

Use `pnpm tauri:build` only when the console shell or desktop host integration changed. For console package work, `pnpm typecheck` and `pnpm build` are the minimum bar after the cargo checks.
