# Performance checklist

1. Compile with `--release`.
2. Check if `lto` is disabled in `Config.toml` (disabling it, makes time 76% of original).
3. Try profiling (see `profiling.md`).
