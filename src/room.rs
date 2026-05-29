use crate::tile::Tile;
use crate::jacobi;
use std::collections::HashMap;
use std::fs;

pub struct Room {
    pub name: String,
    pub tiles: Vec<Tile>,
}

impl Room {
    pub fn load_dir(path: &str) -> Result<Room, String> {
        let dir = fs::read_dir(path).map_err(|e| format!("read_dir: {}", e))?;
        let mut tiles = Vec::new();
        let mut room_name = String::new();
        for entry in dir {
            let entry = entry.map_err(|e| format!("dir entry: {}", e))?;
            let p = entry.path();
            if p.extension().map(|e| e == "md").unwrap_or(false) {
                let md = fs::read_to_string(&p).map_err(|e| format!("read {}: {}", p.display(), e))?;
                let tile = Tile::from_markdown(&md)?;
                if room_name.is_empty() {
                    room_name = tile.room.clone();
                }
                tiles.push(tile);
            }
        }
        Ok(Room { name: room_name, tiles })
    }

    pub fn get(&self, id: &str) -> Option<&Tile> {
        self.tiles.iter().find(|t| t.id == id)
    }

    pub fn insert(&mut self, tile: Tile) {
        if let Some(existing) = self.tiles.iter_mut().find(|t| t.id == tile.id) {
            *existing = tile;
        } else {
            self.tiles.push(tile);
        }
    }

    pub fn what_changed(&self, known_ids: &[String]) -> Vec<&Tile> {
        self.tiles.iter().filter(|t| !known_ids.contains(&t.id)).collect()
    }

    /// Read failures FIRST.
    pub fn failures(&self) -> Vec<&Tile> {
        self.tiles.iter().filter(|t| t.is_failure()).collect()
    }

    pub fn verified(&self) -> Vec<&Tile> {
        self.tiles.iter().filter(|t| t.is_verified()).collect()
    }

    /// Tiles whose dependencies include at least one conjecture.
    pub fn frontier(&self) -> Vec<&Tile> {
        self.tiles.iter().filter(|t| {
            t.dependencies.iter().any(|dep| {
                self.get(dep).map(|d| matches!(d.status, crate::tile::TileStatus::Conjecture)).unwrap_or(false)
            })
        }).collect()
    }

    /// Dependencies that reference non-existent tiles.
    pub fn gaps(&self) -> Vec<String> {
        let ids: Vec<&str> = self.tiles.iter().map(|t| t.id.as_str()).collect();
        let mut missing = Vec::new();
        for t in &self.tiles {
            for dep in &t.dependencies {
                if !ids.contains(&dep.as_str()) && !missing.contains(dep) {
                    missing.push(dep.clone());
                }
            }
        }
        missing
    }

    /// Build dependency adjacency matrix (symmetric, weighted by confidence).
    pub fn dependency_graph(&self) -> Vec<Vec<f64>> {
        let n = self.tiles.len();
        let mut adj = vec![vec![0.0; n]; n];
        let idx: HashMap<&str, usize> = self.tiles.iter().enumerate().map(|(i, t)| (t.id.as_str(), i)).collect();

        for (i, tile) in self.tiles.iter().enumerate() {
            for dep in &tile.dependencies {
                if let Some(&j) = idx.get(dep.as_str()) {
                    let w = (tile.confidence + self.tiles[j].confidence) / 2.0;
                    adj[i][j] = w;
                    adj[j][i] = w;
                }
            }
            // Self-connection with own confidence
            adj[i][i] = tile.confidence;
        }
        adj
    }

    /// Conservation ratio of the knowledge graph.
    pub fn conservation(&self) -> f64 {
        let graph = self.dependency_graph();
        jacobi::conservation_ratio(&graph)
    }

    /// Critical tiles by eigenvector centrality (descending).
    pub fn critical_tiles(&self) -> Vec<(String, f64)> {
        let n = self.tiles.len();
        if n == 0 { return vec![]; }
        let graph = self.dependency_graph();
        let eigs = jacobi::eigenvalues(&graph);
        let leading = eigs.first().copied().unwrap_or(1.0);

        // Use row-sum as proxy for centrality weighted by leading eigenvalue
        let mut scores: Vec<(String, f64)> = self.tiles.iter().enumerate().map(|(i, t)| {
            let row_sum: f64 = graph[i].iter().sum();
            (t.id.clone(), row_sum * leading)
        }).collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }

    pub fn suggest_next(&self) -> Option<String> {
        // Priority: failures first, then frontier, then gaps
        if let Some(f) = self.failures().first() {
            return Some(f.id.clone());
        }
        if let Some(f) = self.frontier().first() {
            return Some(f.id.clone());
        }
        let gaps = self.gaps();
        if let Some(g) = gaps.first() {
            return Some(g.clone());
        }
        None
    }
}
