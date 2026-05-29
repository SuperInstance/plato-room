# plato-room — Knowledge as Spectral Graph

Knowledge rooms where tiles have dependencies, status, and confidence. The room computes its own conservation ratio — a spectral measure of how well-connected the knowledge is. Pure Rust, zero dependencies.

**Part of the [Plato](https://github.com/SuperInstance/plato-shell) ecosystem.**

## What This Gives You

- **Tiles with dependencies** — markdown + YAML front-matter, typed (Fact, Proof, Failure, Benchmark, Code, etc.)
- **Conservation ratio** — Laplacian eigenvalues tell you how well-connected the knowledge is
- **Dependency DAG** — tiles reference each other; the room resolves the graph
- **Failure tiles** — first-class failures with reasons, not just missing success
- **Next-tile suggestions** — failures first, then frontier tiles, then gaps

## Quick Start

```rust
use plato_room::{Room, Tile};

let mut room = Room::new("graph-theory");

// Add tiles (or load from markdown files)
room.add_tile(Tile::from_markdown(r#"
---
id: fourier-basics
kind: Fact
status: Verified
confidence: 0.95
dependencies: complex-numbers, euler-formula
---
Fourier transform decomposes signals into frequency components.
"#));

// Conservation ratio
let cr = room.conservation_ratio();
println!("Room CR: {:.4}", cr);

// What should I work on next?
let next = room.suggest_next();
```

### Tile Format

```markdown
---
id: fourier-basics
room: signal-processing
kind: Fact
status: Verified
title: Fourier transform decomposes signals into frequency components
confidence: 0.95
dependencies: complex-numbers, euler-formula
tags: math, transforms, frequency
---
The Fourier transform converts a time-domain signal into its frequency-domain representation.
```

**Tile kinds:** Fact, Proof, Failure, Benchmark, Code, Observation, Conjecture, Retracted
**Tile status:** Verified, Partial, Failed, Conjecture, Retracted

## How It Fits

The knowledge substrate. [plato-loader](https://github.com/SuperInstance/plato-loader) reads tiles from rooms. [plato-memory](https://github.com/SuperInstance/plato-memory) stores loaded knowledge for agents. The conservation ratio guides what to verify next.

## Installation

```toml
[dependencies]
plato-room = "0.1"
```

## Testing

```bash
cargo test
```

## License

MIT
