use crate::room::Room;
use crate::jacobi;

pub struct KnowledgeBase {
    pub rooms: Vec<Room>,
}

impl KnowledgeBase {
    pub fn new() -> KnowledgeBase {
        KnowledgeBase { rooms: vec![] }
    }

    pub fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }

    /// Global conservation ratio across all rooms' dependency graphs.
    pub fn conservation(&self) -> f64 {
        if self.rooms.is_empty() { return 0.0; }
        let mut total_size = 0;
        let mut combined: Vec<Vec<f64>> = vec![];
        for room in &self.rooms {
            let g = room.dependency_graph();
            let offset = total_size;
            let n = g.len();
            // Extend combined matrix
            let new_size = total_size + n;
            for row in &mut combined {
                row.resize(new_size, 0.0);
            }
            for i in 0..n {
                let mut new_row = vec![0.0; new_size];
                for j in 0..n {
                    new_row[offset + j] = g[i][j];
                }
                combined.push(new_row);
            }
            total_size = new_size;
        }
        if combined.is_empty() { return 0.0; }
        jacobi::conservation_ratio(&combined)
    }

    /// Alignment between two rooms: cosine similarity of their eigenvalue spectra.
    pub fn room_alignment(&self, a: &str, b: &str) -> f64 {
        let ra = self.rooms.iter().find(|r| r.name == a);
        let rb = self.rooms.iter().find(|r| r.name == b);
        match (ra, rb) {
            (Some(ra), Some(rb)) => {
                let ea = jacobi::eigenvalues(&ra.dependency_graph());
                let eb = jacobi::eigenvalues(&rb.dependency_graph());
                let max_len = ea.len().max(eb.len());
                if max_len == 0 { return 0.0; }
                let mut dot = 0.0_f64;
                let mut norm_a = 0.0_f64;
                let mut norm_b = 0.0_f64;
                for i in 0..max_len {
                    let va = ea.get(i).copied().unwrap_or(0.0);
                    let vb = eb.get(i).copied().unwrap_or(0.0);
                    dot += va * vb;
                    norm_a += va * va;
                    norm_b += vb * vb;
                }
                let denom = norm_a.sqrt() * norm_b.sqrt();
                if denom < 1e-30 { 0.0 } else { dot / denom }
            }
            _ => 0.0,
        }
    }

    /// Pairs of rooms with low alignment — missing bridges.
    pub fn missing_bridges(&self) -> Vec<(String, String)> {
        let mut bridges = Vec::new();
        for i in 0..self.rooms.len() {
            for j in (i + 1)..self.rooms.len() {
                let align = self.room_alignment(&self.rooms[i].name, &self.rooms[j].name);
                if align < 0.5 {
                    bridges.push((self.rooms[i].name.clone(), self.rooms[j].name.clone()));
                }
            }
        }
        bridges
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self { Self::new() }
}
