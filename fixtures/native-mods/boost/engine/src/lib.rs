//! The bundle's own crate: built as a workspace member, separately from the game crate.
const TABLE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/table.json"));

pub fn boost(score: i64) -> i64 {
    let table: serde_json::Value = serde_json::from_slice(TABLE).unwrap();
    score + table["bonus"].as_i64().unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn bonus_comes_from_the_crate_data() {
        assert_eq!(super::boost(2), 42);
    }
}
