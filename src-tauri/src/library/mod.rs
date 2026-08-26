//! Phase 2 groundwork: SQLite persistence + library scan pipeline.
//!
//! M1: db open/migrations/settings. M2: scanner (`scan.rs`) feeding the same
//! schema. Design contract lives in PHASE2.md.

pub mod artwork;
pub mod db;
pub mod import;
pub mod scan;
pub mod settings;
pub mod tags;

/// Stable text ID from natural keys ("ar-hex", "al-hex", "tr-hex"). Same
/// inputs always yield the same ID, so rescans upsert instead of duplicating.
pub fn stable_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(&[0]);
    }
    let hash = hasher.finalize();
    format!("{prefix}-{}", &hash.to_hex()[..16])
}

/// Rust port of `src/lib/sort.ts` sortKey: trim, lowercase, strip a leading
/// "the ", fold accents to base letters (both precomposed and decomposed
/// forms). Used for artist/album `sort_name` at insert time.
pub fn sort_key(name: &str) -> String {
    let trimmed = name.trim().to_lowercase();
    let stripped = trimmed.strip_prefix("the ").unwrap_or(&trimmed);
    stripped
        .chars()
        .filter(|c| !matches!(*c as u32, 0x0300..=0x036F)) // combining marks
        .map(|c| FOLD_MAP.get(&c).copied().unwrap_or(c))
        .collect()
}

/// Precomposed → base letter map for the Latin-1/Latin-Extended ranges that
/// dominate music metadata. Lowercase-only; callers lowercase first.
static FOLD_MAP: std::sync::LazyLock<std::collections::HashMap<char, char>> =
    std::sync::LazyLock::new(|| {
        [
            ('à','a'),('á','a'),('â','a'),('ã','a'),('ä','a'),('å','a'),
            ('è','e'),('é','e'),('ê','e'),('ë','e'),
            ('ì','i'),('í','i'),('î','i'),('ï','i'),
            ('ò','o'),('ó','o'),('ô','o'),('õ','o'),('ö','o'),('ø','o'),
            ('ù','u'),('ú','u'),('û','u'),('ü','u'),
            ('ý','y'),('ÿ','y'),('ñ','n'),('ç','c'),('ð','d'),('đ','d'),
            ('ģ','g'),('ķ','k'),('ļ','l'),('ņ','n'),('š','s'),('ž','z'),
            ('ţ','t'),('ŗ','r'),('ē','e'),('ī','i'),('ō','o'),('ū','u'),
        ]
        .into_iter()
        .collect()
    });

#[cfg(test)]
mod tests {
    #[test]
    fn sort_key_mirrors_typescript_sortkey() {
        assert_eq!(super::sort_key("  The Birthday Massacre "), "birthday massacre");
        assert_eq!(super::sort_key("Björk"), "bjork");
        assert_eq!(super::sort_key("Motörhead"), "motorhead");
        // Decomposed (NFD) input folds identically.
        assert_eq!(super::sort_key("Zel\u{301}va"), "zelva");
        assert_eq!(super::sort_key("Sigur Rós"), "sigur ros");
    }

    #[test]
    fn stable_ids_are_deterministic_and_prefix_separated() {
        let a = super::stable_id("ar", &["helloween"]);
        let b = super::stable_id("ar", &["helloween"]);
        let c = super::stable_id("ar", &["hello", "ween"]);
        let d = super::stable_id("tr", &["helloween"]);
        assert_eq!(a, b);
        assert_ne!(a, c); // separator prevents part-boundary collisions
        assert_ne!(a, d);
        assert!(a.starts_with("ar-"));
        assert_eq!(a.len(), "ar-".len() + 16);
    }

    #[test]
    fn different_parts_differ() {
        assert_ne!(
            super::stable_id("al", &["helloween", "Giants & Monsters", "2021"]),
            super::stable_id("al", &["helloween", "Giants & Monsters", "2022"])
        );
    }
}
