# Fishbait Chess Engine
A Rust chess engine based on the [shakmaty](https://github.com/niklasf/shakmaty) chess library.

Implemented using:
- Minimax with Alpha-Beta Pruning
- Iterative Deepening
- Late Move Reduction
- Quiescence Search
- Transposition Table with Zobrist hashing
- Move ordering with:
  - MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
  - Silent move history
