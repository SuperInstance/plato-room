# plato-room

**Knowledge as spectral graph. Tiles with dependencies. The room IS the intelligence.**

Pure Rust, zero dependencies. Knowledge gets a conservation ratio. Well-connected knowledge survives. Disconnected facts get lost.

## The problem

Knowledge decays between sessions. Files lose context. A folder of markdown notes is a pile of facts — no structure, no dependencies, no way to know what matters or what's missing.

When you come back to a project after weeks away, you don't need all the facts. You need the *structure* — what depends on what, what's verified, what failed, where the gaps are. Plain files don't give you that.

PLATO rooms do. Each room is a knowledge graph where every tile has dependencies, status, and confidence. The room computes its own conservation ratio — a spectral measure of how well-connected the knowledge is. Well-connected knowledge (high CR) survives context switches. Disconnected facts (low CR) get lost.

## The core idea

Knowledge has a conservation ratio. Build a dependency graph from your tiles. Compute the Laplacian eigenvalues. The ratio of the leading eigenvalue to the total tells you how concentrated your knowledge is — how much of the structure is held together by a few critical connections vs. spread across many independent facts.

High CR: knowledge is tightly connected. Changing one tile ripples through the room. This is good for deep domains but fragile.

Low CR: knowledge is scattered. Tiles don't depend on each other. Easy to lose pieces without noticing.

The room uses this to tell you what to work on next — failures first, then frontier tiles, then gaps.

## Tile format

Tiles are markdown with YAML front-matter:

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
The Fourier transform converts a time-domain signal into its frequency-domain
representation. For a continuous signal f(t):

F(ω) = ∫ f(t)·e^(-iωt) dt

This depends on understanding complex exponentials (Euler's formula).
```

Tile kinds: `Fact`, `Proof`, `Failure`, `Benchmark`, `Code`, `Observation`, `Conjecture`, `Retracted`

Tile status: `Verified`, `Partial`, `Failed`, `Conjecture`, `Retracted`

### The failure tile

```markdown
---
id: naive-convolution
room: signal-processing
kind: Failure
status: Failed
title: Direct convolution is O(n²) — too slow for real-time
confidence: 0.0
dependencies: fourier-basics
tags: performance, failure
---
Tried direct time-domain convolution for a 44.1kHz audio buffer.
Takes ~4ms per buffer — exceeds the 2ms budget.

The FFT-based approach (convolution theorem) is the way to go.
This failure is documented so nobody tries it again.
```

Failures are first-class knowledge. They're often more valuable than successes.

## Using a room

### Load tiles from a directory

```rust
use plato_room::room::Room;

let room = Room::load_dir("./tiles/signal-processing")?;
println!("Room: {} ({} tiles)", room.name, room.tiles.len());
```

Or build programmatically:

```rust
use plato_room::tile::{Tile, TileKind, TileStatus};
use plato_room::room::Room;

let mut room = Room { name: "math".into(), tiles: vec![] };

room.insert(Tile {
    id: "pythagoras".into(),
    room: "math".into(),
    kind: TileKind::Fact,
    status: TileStatus::Verified,
    title: "a² + b² = c² for right triangles".into(),
    content: "Proven by area decomposition...".into(),
    dependencies: vec!["triangles".into()],
    confidence: 1.0,
    tags: vec!["geometry".into()],
});
```

### The failure-first principle

```rust
// Read what didn't work before what did
let failures = room.failures();
for f in &failures {
    println!("FAILED: {} — {}", f.id, f.title);
}

// Then verified knowledge
let verified = room.verified();
```

When you enter a room, read failures first. They prevent you from repeating mistakes. Then verified facts. Then frontier tiles (things that depend on conjectures). This is how human experts actually work — they know the dead ends by heart.

### Check the structure

```rust
// What's missing? Dependencies that reference non-existent tiles
let gaps = room.gaps();
for g in &gaps {
    println!("Missing tile: {}", g);
}

// What tiles depend on unproven conjectures?
let frontier = room.frontier();

// How well-connected is this knowledge?
let cr = room.conservation();
println!("Conservation ratio: {:.4}", cr);

// Which tiles are most critical (by eigenvector centrality)?
let critical = room.critical_tiles();
for (id, score) in &critical {
    println!("{}: centrality = {:.4}", id, score);
}
```

### suggest_next(): the room tells you what to write

```rust
// Priority: failures → frontier → gaps → None (room is complete)
if let Some(next) = room.suggest_next() {
    println!("Work on: {}", next);
}
```

The room has an opinion about what you should do next. It's not random — it follows the failure-first principle, then frontier tiles (advancing from conjectures to proofs), then filling in gaps.

## KnowledgeBase: multi-room conservation

```rust
use plato_room::knowledge_base::KnowledgeBase;

let mut kb = KnowledgeBase::new();
kb.add_room(signal_processing);
kb.add_room(linear_algebra);

// Global conservation across all rooms
let global_cr = kb.conservation();

// How aligned are two rooms structurally?
let alpha = kb.room_alignment("signal-processing", "linear-algebra");

// Find rooms that need bridges between them
let bridges = kb.missing_bridges();
for (a, b) in &bridges {
    println!("Low alignment: {} ↔ {} — add cross-references", a, b);
}
```

The `KnowledgeBase` treats all rooms as one big block-diagonal graph and computes conservation across the whole thing. Rooms with low alignment are separate silos — they need bridge tiles that reference dependencies across rooms.

## The math

- **Dependency graph**: adjacency matrix where `A[i][j] = (confidence_i + confidence_j) / 2` if tile i depends on tile j.
- **Eigenvalues**: computed via Jacobi rotation (pure Rust, converges to 1e-14).
- **Conservation ratio**: `λ₁ / Σλᵢ` — the leading eigenvalue divided by the total. Measures how concentrated the graph structure is.
- **Room alignment**: cosine similarity of eigenvalue spectra between two rooms' dependency graphs. Same spectral technique as `conservation-protocol`.

## Honest limitations

1. **Requires disciplined tile writing.** The system only works if you actually write tiles with proper dependencies, confidence scores, and statuses. A room full of low-quality tiles gives you a low-quality graph. There's no shortcut.

2. **CR is only one quality metric.** A high conservation ratio doesn't mean the knowledge is *correct* — it means it's well-connected. You can have a tightly connected web of wrong facts.

3. **YAML parsing is simplistic.** The front-matter parser handles `key: value` pairs and comma-separated lists. It's not a full YAML parser. No nested structures, no multiline values, no quoting beyond what simple split-on-colon provides.

4. **Jacobi eigenvalue is O(n³).** Fine for rooms with dozens or hundreds of tiles. Not suitable for rooms with tens of thousands without switching to iterative methods.

5. **No persistence layer.** Rooms load from a directory of markdown files and exist in memory. There's no database, no indexing, no incremental loading. This is intentional — the files are the database. Use git for history.

## Running

```bash
cargo run
```

Minimal main — just prints the banner. The real value is in the library.

## Testing

```bash
cargo test
```

Integration tests cover tile parsing roundtrips, room operations (insert, get, what_changed), failure-first ordering, frontier detection, gap detection, dependency graph construction, conservation ratio, critical tile sorting, suggest_next priority, multi-room knowledge bases, and Jacobi eigenvalue correctness.

## Architecture

```
tile.rs          — Tile struct, front-matter parsing, kind/status enums
room.rs          — Room: load tiles, failures, verified, frontier, gaps, conservation, suggest_next
knowledge_base.rs — Multi-room conservation, room alignment, missing bridges
jacobi.rs        — Jacobi eigenvalue decomposition, conservation ratio
```

## License

MIT
