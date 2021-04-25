# How to use turbocow ui

## Push Mode
1. Start ui server using:
```
cargo run --release ui
```
It starts a window an server to receive images. Keep it running.

2. Render your images as usual running the following command in a parallel terminal session:
```
cargo run --release render
```