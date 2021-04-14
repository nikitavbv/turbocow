# Profiling

Record profile with:
```
perf record --call-graph=dwarf ./target/release/core
```

View profile with:
```
perf report --hierarchy -M intel
```