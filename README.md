# Fishbait Chess Engine

A simple Rust chess engine based on the [shakmaty](https://github.com/niklasf/shakmaty) chess library.

Implements:

- Minimax with:
  - Alpha-Beta Pruning
  - Iterative Deepening
  - Aspiration window
  - Late Move Reduction
  - Quiescence Search
  - Transposition Table with Zobrist hashing
  - Move ordering with:
    - MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    - Silent move history

## UCI compatibility

Playable in any UCI-supporting GUI such as XBoard.

Compile with:

```
cargo build --release
```

Use the resulting binary in /target with your favourite GUI.
