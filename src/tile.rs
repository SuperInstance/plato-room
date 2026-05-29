use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TileKind {
    Fact,
    Proof,
    Failure,
    Benchmark,
    Code,
    Observation,
    Conjecture,
    Retracted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileStatus {
    Verified,
    Partial,
    Failed,
    Conjecture,
    Retracted,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub id: String,
    pub room: String,
    pub kind: TileKind,
    pub status: TileStatus,
    pub title: String,
    pub content: String,
    pub dependencies: Vec<String>,
    pub confidence: f64,
    pub tags: Vec<String>,
}

impl Tile {
    pub fn from_markdown(md: &str) -> Result<Tile, String> {
        let mut lines = md.lines();
        // Expect opening ---
        let first = lines.next().ok_or("empty input")?;
        if first.trim() != "---" {
            return Err("expected opening ---".into());
        }
        let mut front: Vec<String> = Vec::new();
        let mut found_close = false;
        for line in &mut lines {
            if line.trim() == "---" {
                found_close = true;
                break;
            }
            front.push(line.to_string());
        }
        if !found_close {
            return Err("expected closing ---".into());
        }
        let front_text = front.join("\n");
        let meta = parse_yaml_simple(&front_text)?;

        let id = meta.get("id").cloned().ok_or("missing id")?;
        let room = meta.get("room").cloned().unwrap_or_default();
        let title = meta.get("title").cloned().unwrap_or_default();
        let kind_str = meta.get("kind").cloned().ok_or("missing kind")?;
        let kind = parse_kind(&kind_str)?;
        let status_str = meta.get("status").cloned().ok_or("missing status")?;
        let status = parse_status(&status_str)?;
        let confidence: f64 = meta.get("confidence").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let dependencies = meta.get("dependencies")
            .map(|s| s.split(',').map(|d| d.trim().to_string()).filter(|d| !d.is_empty()).collect())
            .unwrap_or_default();
        let tags = meta.get("tags")
            .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
            .unwrap_or_default();

        let content = lines.collect::<Vec<&str>>().join("\n").trim().to_string();

        Ok(Tile { id, room, kind, status, title, content, dependencies, confidence, tags })
    }

    pub fn to_markdown(&self) -> String {
        let deps = self.dependencies.join(", ");
        let tags = self.tags.join(", ");
        format!(
            "---\nid: {}\nroom: {}\nkind: {}\nstatus: {}\ntitle: {}\nconfidence: {}\ndependencies: {}\ntags: {}\n---\n{}",
            self.id,
            self.room,
            kind_str(&self.kind),
            status_str(&self.status),
            self.title,
            self.confidence,
            deps,
            tags,
            self.content,
        )
    }

    pub fn is_failure(&self) -> bool {
        matches!(self.kind, TileKind::Failure) || matches!(self.status, TileStatus::Failed)
    }

    pub fn is_verified(&self) -> bool {
        matches!(self.status, TileStatus::Verified)
    }
}

fn parse_kind(s: &str) -> Result<TileKind, String> {
    match s.trim().to_lowercase().as_str() {
        "fact" => Ok(TileKind::Fact),
        "proof" => Ok(TileKind::Proof),
        "failure" => Ok(TileKind::Failure),
        "benchmark" => Ok(TileKind::Benchmark),
        "code" => Ok(TileKind::Code),
        "observation" => Ok(TileKind::Observation),
        "conjecture" => Ok(TileKind::Conjecture),
        "retracted" => Ok(TileKind::Retracted),
        _ => Err(format!("unknown kind: {}", s)),
    }
}

fn kind_str(k: &TileKind) -> &'static str {
    match k {
        TileKind::Fact => "Fact",
        TileKind::Proof => "Proof",
        TileKind::Failure => "Failure",
        TileKind::Benchmark => "Benchmark",
        TileKind::Code => "Code",
        TileKind::Observation => "Observation",
        TileKind::Conjecture => "Conjecture",
        TileKind::Retracted => "Retracted",
    }
}

fn parse_status(s: &str) -> Result<TileStatus, String> {
    match s.trim().to_lowercase().as_str() {
        "verified" => Ok(TileStatus::Verified),
        "partial" => Ok(TileStatus::Partial),
        "failed" => Ok(TileStatus::Failed),
        "conjecture" => Ok(TileStatus::Conjecture),
        "retracted" => Ok(TileStatus::Retracted),
        _ => Err(format!("unknown status: {}", s)),
    }
}

fn status_str(s: &TileStatus) -> &'static str {
    match s {
        TileStatus::Verified => "Verified",
        TileStatus::Partial => "Partial",
        TileStatus::Failed => "Failed",
        TileStatus::Conjecture => "Conjecture",
        TileStatus::Retracted => "Retracted",
    }
}

fn parse_yaml_simple(text: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_string();
            let val = line[pos + 1..].trim().to_string();
            map.insert(key, val);
        }
    }
    Ok(map)
}
