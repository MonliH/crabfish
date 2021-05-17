# crabfish 🦀♟️

Crabfish is a **chess engine** written from scratch, in rust. 
It can provide a **strong next move** for the current player, or an **evaluation of a board position**.

## Install
```
cargo install crabfish
```

## Build From Source

```
git clone https://github.com/MonliH/crabfish.git
cd crabfish
cargo run --release
```

Note: the `--release` flag when building is **VERY IMPORTANT**.
The engine can not search very deep without the optimizations provided by it.
