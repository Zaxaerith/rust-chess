//! Bounded JSON content storage and explicit backup import policies.
use crate::{
    persistence,
    repertoire::{Collection, MAX_BYTES},
};
use std::{collections::BTreeSet, fs, io::Read, path::Path};

fn read(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Repertoire metadata {}: {e}", path.display())),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Repertoire data must be a regular file; preserved".into());
    }
    if metadata.len() > MAX_BYTES as u64 {
        return Err("Repertoire JSON exceeds 8 MiB; preserved".into());
    }
    let mut bytes = vec![];
    fs::File::open(path)
        .map_err(|e| format!("Repertoire read: {e}"))?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Repertoire read: {e}"))?;
    if bytes.len() > MAX_BYTES {
        return Err("Repertoire JSON exceeds 8 MiB; preserved".into());
    }
    Ok(Some(bytes))
}
pub fn load(path: &Path) -> Result<Collection, String> {
    let Some(bytes) = read(path)? else {
        return Ok(Collection::default());
    };
    decode(&bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Collection, String> {
    if bytes.len() > MAX_BYTES {
        return Err("Repertoire JSON exceeds 8 MiB".into());
    }
    let c: Collection = serde_json::from_slice(bytes)
        .map_err(|e| format!("Repertoire JSON damaged or incompatible; preserve original: {e}"))?;
    c.validate()?;
    Ok(c)
}
pub fn encode(collection: &Collection) -> Result<Vec<u8>, String> {
    collection.validate()?;
    let bytes =
        serde_json::to_vec_pretty(collection).map_err(|e| format!("Repertoire JSON: {e}"))?;
    if bytes.len() > MAX_BYTES {
        return Err("Repertoire JSON exceeds 8 MiB".into());
    }
    Ok(bytes)
}
pub fn save(path: &Path, collection: &Collection) -> Result<(), String> {
    let bytes = encode(collection)?;
    // A corrupt/unknown existing primary is never overwritten with an empty/default collection.
    load(path)?;
    persistence::atomic_write(path, &bytes)
}
pub fn export(path: &Path, collection: &Collection) -> Result<(), String> {
    // Backup selection/overwrite confirmation belongs to the native save dialog. The caller
    // also excludes managed app files. Existing JSON must nevertheless be readable/valid.
    let bytes = encode(collection)?;
    load(path)?;
    persistence::atomic_write(path, &bytes)
}
fn unique_id(prefix: &str, used: &mut BTreeSet<String>) -> String {
    loop {
        let id = crate::repertoire::new_id(prefix);
        if used.insert(id.clone()) {
            return id;
        }
    }
}
fn used_ids(collection: &Collection) -> BTreeSet<String> {
    collection
        .repertoires
        .iter()
        .flat_map(|book| {
            std::iter::once(book.id.clone()).chain(book.routes.iter().map(|r| r.id.clone()))
        })
        .collect()
}
pub fn import_new(
    current: &Collection,
    incoming: &Collection,
    now: u64,
) -> Result<Collection, String> {
    current.validate()?;
    incoming.validate()?;
    let mut next = current.clone();
    let mut ids = used_ids(current);
    // Construct in one pass, then validate once. A thousand-route import must not
    // repeatedly re-parse every already imported route (quadratic chess work).
    for original in &incoming.repertoires {
        let mut book = original.clone();
        book.id = unique_id("b", &mut ids);
        book.created_unix = now;
        book.modified_unix = now;
        for route in &mut book.routes {
            route.id = unique_id("r", &mut ids);
        }
        next.repertoires.push(book);
    }
    next.validate()?;
    Ok(next)
}
pub fn merge(
    current: &Collection,
    incoming: &Collection,
    target: &str,
    now: u64,
) -> Result<Collection, String> {
    current.validate()?;
    incoming.validate()?;
    let side = current.get(target).ok_or("Target repertoire missing")?.side;
    if incoming.repertoires.iter().any(|b| b.side != side) {
        return Err(
            "Merge side differs from target; choose New independent repertoires instead".into(),
        );
    }
    let mut next = current.clone();
    let mut ids = used_ids(current);
    let book = next.get_mut(target).ok_or("Target repertoire missing")?;
    let mut paths: BTreeSet<Vec<String>> = book.routes.iter().map(|r| r.moves.clone()).collect();
    let before = book.routes.len();
    for source in &incoming.repertoires {
        for original in &source.routes {
            if paths.insert(original.moves.clone()) {
                let mut route = original.clone();
                route.id = unique_id("r", &mut ids);
                book.routes.push(route);
            }
        }
    }
    if book.routes.len() != before {
        book.modified_unix = now.max(book.modified_unix);
    }
    next.validate()?;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repertoire::Side;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    fn temp() -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::current_dir()
            .unwrap()
            .join("target/v028-qa/store")
            .join(format!(
                "{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn sample() -> Collection {
        let mut c = Collection::default();
        let id = c.create("My white 开局", Side::White, 1).unwrap();
        c.rename(&id, "My white 开局", "说明\nUnicode", 2).unwrap();
        c.add_route(
            &id,
            vec!["e2e4".into(), "e7e5".into()],
            "line",
            "route annotation",
            3,
        )
        .unwrap();
        c
    }
    #[test]
    fn json_roundtrip_independent_import_merge_and_restart() {
        let root = temp();
        let primary = root.join("repertoires-v1.json");
        let backup = root.join("backup.json");
        let c = sample();
        save(&primary, &c).unwrap();
        assert_eq!(load(&primary).unwrap(), c);
        export(&backup, &c).unwrap();
        assert_eq!(load(&backup).unwrap(), c);
        let added = import_new(&c, &load(&backup).unwrap(), 10).unwrap();
        assert_eq!(added.repertoires.len(), 2);
        assert_ne!(added.repertoires[0].id, added.repertoires[1].id);
        assert_eq!(added.repertoires[1].note, c.repertoires[0].note);
        assert_ne!(
            added.repertoires[1].routes[0].id,
            c.repertoires[0].routes[0].id
        );
        let merged = merge(&c, &c, &c.repertoires[0].id, 10).unwrap();
        assert_eq!(merged, c);
        let mut black = Collection::default();
        black.create("Black", Side::Black, 0).unwrap();
        assert!(merge(&c, &black, &c.repertoires[0].id, 10).is_err());
    }
    #[test]
    fn damaged_unknown_schema_and_disk_failures_preserve_primary() {
        let root = temp();
        let primary = root.join("repertoires-v1.json");
        let c = sample();
        for bad in [
            b"damaged".as_slice(),
            b"{\"schema_version\":2,\"repertoires\":[]}".as_slice(),
        ] {
            fs::write(&primary, bad).unwrap();
            assert!(load(&primary).is_err());
            assert!(save(&primary, &c).is_err());
            assert_eq!(fs::read(&primary).unwrap(), bad);
        }
        let blocked = root.join("blocked.json");
        fs::create_dir(&blocked).unwrap();
        assert!(save(&blocked, &c).is_err());
        assert!(blocked.is_dir());
        assert!(decode(&vec![b' '; MAX_BYTES + 1]).is_err());
        assert!(decode(b"{\"schema_version\":1,\"repertoires\":[],\"unknown\":true}").is_err());
    }
}
