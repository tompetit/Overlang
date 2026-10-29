//! The content-addressed store (spec section 10): a SQLite database at
//! `.ovstore/store.sqlite` under the project root.

use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub type R<T> = rusqlite::Result<T>;

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub hash: String,
    pub kind: String,
    pub ast: String,
    pub source: String,
    pub created_at: i64,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS defs(hash TEXT PRIMARY KEY, kind TEXT, ast TEXT, source TEXT, created_at INTEGER);
CREATE TABLE IF NOT EXISTS names(name TEXT PRIMARY KEY, hash TEXT, updated_at INTEGER);
CREATE TABLE IF NOT EXISTS history(name TEXT, hash TEXT, at INTEGER);
CREATE TABLE IF NOT EXISTS deps(hash TEXT, dep_hash TEXT, PRIMARY KEY(hash, dep_hash));
CREATE TABLE IF NOT EXISTS cache(hash TEXT, kind TEXT, blob BLOB, at INTEGER, PRIMARY KEY(hash, kind));
CREATE INDEX IF NOT EXISTS deps_by_dep ON deps(dep_hash);
";

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

impl Store {
    /// Open `<root>/.ovstore/store.sqlite`, creating it on first use.
    pub fn open(root: &Path) -> Result<Store, String> {
        let dir = root.join(".ovstore");
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        let conn = Connection::open(dir.join("store.sqlite")).map_err(|e| e.to_string())?;
        conn.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
        Ok(Store { conn })
    }

    pub fn begin(&self) -> R<()> {
        self.conn.execute_batch("BEGIN")
    }

    pub fn commit(&self) -> R<()> {
        self.conn.execute_batch("COMMIT")
    }

    /// Store a definition; idempotent by hash. An existing row keeps its
    /// canonical form and only takes the newer `source` text (a rename).
    /// Returns true if the row is new.
    pub fn put(&self, hash: &str, kind: &str, ast: &str, source: &str, deps: &[String]) -> R<bool> {
        let added = self.conn.execute(
            "INSERT OR IGNORE INTO defs(hash, kind, ast, source, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![hash, kind, ast, source, now()],
        )? == 1;
        if !added {
            self.conn.execute("UPDATE defs SET source = ?2 WHERE hash = ?1 AND source != ?2", params![hash, source])?;
        }
        for d in deps {
            self.conn.execute("INSERT OR IGNORE INTO deps(hash, dep_hash) VALUES (?1, ?2)", params![hash, d])?;
        }
        Ok(added)
    }

    /// Point `name` at `hash`, appending to its history when that changes it.
    pub fn bind(&self, name: &str, hash: &str) -> R<()> {
        if self.hash_of_name(name)?.as_deref() == Some(hash) {
            return Ok(());
        }
        let at = now();
        self.conn.execute("INSERT OR REPLACE INTO names(name, hash, updated_at) VALUES (?1, ?2, ?3)", params![name, hash, at])?;
        self.conn.execute("INSERT INTO history(name, hash, at) VALUES (?1, ?2, ?3)", params![name, hash, at])?;
        Ok(())
    }

    /// Remove `name`; returns the hash it pointed to.
    pub fn unbind(&self, name: &str) -> R<Option<String>> {
        let hash = self.hash_of_name(name)?;
        self.conn.execute("DELETE FROM names WHERE name = ?1", params![name])?;
        Ok(hash)
    }

    pub fn hash_of_name(&self, name: &str) -> R<Option<String>> {
        self.conn.query_row("SELECT hash FROM names WHERE name = ?1", params![name], |r| r.get(0)).optional()
    }

    /// Names currently bound to `hash`, most recently bound first.
    pub fn names_of(&self, hash: &str) -> R<Vec<String>> {
        self.strings("SELECT name FROM names WHERE hash = ?1 ORDER BY updated_at DESC, name", hash)
    }

    pub fn get(&self, hash: &str) -> R<Option<Def>> {
        self.conn
            .query_row("SELECT hash, kind, ast, source, created_at FROM defs WHERE hash = ?1", params![hash], |r| {
                Ok(Def { hash: r.get(0)?, kind: r.get(1)?, ast: r.get(2)?, source: r.get(3)?, created_at: r.get(4)? })
            })
            .optional()
    }

    /// Every stored hash starting with `prefix`.
    pub fn by_prefix(&self, prefix: &str) -> R<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT hash FROM defs WHERE substr(hash, 1, length(?1)) = ?1 ORDER BY hash")?;
        stmt.query_map(params![prefix], |r| r.get(0))?.collect()
    }

    /// Bound names as (name, hash, kind), by name, optionally of one kind.
    pub fn names(&self, kind: Option<&str>) -> R<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.name, n.hash, d.kind FROM names n JOIN defs d ON d.hash = n.hash
             WHERE ?1 IS NULL OR d.kind = ?1 ORDER BY n.name",
        )?;
        stmt.query_map(params![kind], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect()
    }

    pub fn deps(&self, hash: &str) -> R<Vec<String>> {
        self.strings("SELECT dep_hash FROM deps WHERE hash = ?1 ORDER BY dep_hash", hash)
    }

    pub fn dependents(&self, hash: &str) -> R<Vec<String>> {
        self.strings("SELECT hash FROM deps WHERE dep_hash = ?1 ORDER BY hash", hash)
    }

    /// Transitive dependencies (or dependents, with `reverse`) of `hash` as
    /// (hash, depth), ordered by depth; depth is the shortest path length.
    pub fn closure(&self, hash: &str, reverse: bool) -> R<Vec<(String, usize)>> {
        let mut seen = HashSet::from([hash.to_string()]);
        let mut queue = VecDeque::from([(hash.to_string(), 0)]);
        let mut out = Vec::new();
        while let Some((h, depth)) = queue.pop_front() {
            let next = if reverse { self.dependents(&h)? } else { self.deps(&h)? };
            for n in next {
                if seen.insert(n.clone()) {
                    out.push((n.clone(), depth + 1));
                    queue.push_back((n, depth + 1));
                }
            }
        }
        Ok(out)
    }

    /// Every hash `name` has pointed to, as (hash, at), newest first.
    pub fn history(&self, name: &str) -> R<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare("SELECT hash, at FROM history WHERE name = ?1 ORDER BY at DESC, rowid DESC")?;
        stmt.query_map(params![name], |r| Ok((r.get(0)?, r.get(1)?)))?.collect()
    }

    pub fn cache_get(&self, hash: &str, kind: &str) -> R<Option<Vec<u8>>> {
        self.conn.query_row("SELECT blob FROM cache WHERE hash = ?1 AND kind = ?2", params![hash, kind], |r| r.get(0)).optional()
    }

    pub fn cache_put(&self, hash: &str, kind: &str, blob: &[u8]) -> R<()> {
        self.conn.execute("INSERT OR REPLACE INTO cache(hash, kind, blob, at) VALUES (?1, ?2, ?3, ?4)", params![hash, kind, blob, now()])?;
        Ok(())
    }

    fn strings(&self, sql: &str, arg: &str) -> R<Vec<String>> {
        let mut stmt = self.conn.prepare(sql)?;
        stmt.query_map(params![arg], |r| r.get(0))?.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_bind_closure_and_cache() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/tmp/store-unit");
        let _ = std::fs::remove_dir_all(&dir);
        let s = Store::open(&dir).unwrap();
        assert!(s.put("aa", "fn", "{}", "fn a", &["bb".into()]).unwrap());
        assert!(!s.put("aa", "fn", "{}", "fn a2", &["bb".into()]).unwrap());
        assert_eq!(s.get("aa").unwrap().unwrap().source, "fn a2");
        s.put("bb", "fn", "{}", "fn b", &["cc".into()]).unwrap();
        s.put("cc", "fn", "{}", "fn c", &[]).unwrap();
        assert_eq!(s.closure("aa", false).unwrap(), vec![("bb".to_string(), 1), ("cc".to_string(), 2)]);
        assert_eq!(s.closure("cc", true).unwrap(), vec![("bb".to_string(), 1), ("aa".to_string(), 2)]);
        s.bind("a", "aa").unwrap();
        s.bind("a", "aa").unwrap();
        s.bind("a", "bb").unwrap();
        assert_eq!(s.history("a").unwrap().iter().map(|h| h.0.as_str()).collect::<Vec<_>>(), ["bb", "aa"]);
        assert_eq!(s.by_prefix("b").unwrap(), ["bb"]);
        assert_eq!(s.unbind("a").unwrap().as_deref(), Some("bb"));
        assert_eq!(s.hash_of_name("a").unwrap(), None);
        s.cache_put("aa", "c", b"x").unwrap();
        assert_eq!(s.cache_get("aa", "c").unwrap().as_deref(), Some(&b"x"[..]));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
