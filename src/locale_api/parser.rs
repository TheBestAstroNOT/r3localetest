use super::sanitizer::sanitize_r3_locale_file;
use super::types::TableEntry;
use super::types::{LocaleTable, ParseR3Error};
use hashbrown::HashTable;
use memchr::{memchr, memmem};
use std::fs;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

pub fn parse_r3locale_file(path: &Path) -> Result<LocaleTable, ParseR3Error> {
    if !path.exists() {
        return Err(ParseR3Error::FileNotFound);
    }
    let bytes = fs::read(path).map_err(|_| ParseR3Error::FailedToRead)?;
    parse_r3locale_bytes(&bytes)
}

//Parses a reloaded 3 localisation file and returns a LocaleTable
pub fn parse_r3locale_bytes(bytes: &[u8]) -> Result<LocaleTable, ParseR3Error> {
    let sanitised_bytes: Box<[u8]> = match sanitize_r3_locale_file(bytes) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };
    let opening_brackets_matches_initial: Vec<usize> =
        memmem::find_iter(&sanitised_bytes, b"[[").collect();
    let mut opening_brackets_matches_final: Vec<usize> =
        Vec::with_capacity(opening_brackets_matches_initial.len());
    let mut closing_brackets_matches_final: Vec<usize> =
        Vec::with_capacity(opening_brackets_matches_initial.len());
    let mut value_start: Vec<usize> = Vec::with_capacity(opening_brackets_matches_initial.len());
    for item in &opening_brackets_matches_initial {
        if *item == 0 || sanitised_bytes[item - 1] == b'\n' {
            opening_brackets_matches_final.push(*item);
            if let Some(close_pos) = memmem::find(&sanitised_bytes[*item..], b"]]") {
                closing_brackets_matches_final.push(item + close_pos);
                if let Some(value_open_pos) = memchr(b'\n', &sanitised_bytes[item + close_pos..]) {
                    value_start.push(item + close_pos + value_open_pos);
                } else {
                    return Err(ParseR3Error::KeyValueMismatch);
                }
            } else {
                return Err(ParseR3Error::BracketMismatch);
            }
        }
    }

    opening_brackets_matches_final.dedup();
    opening_brackets_matches_final.sort();
    closing_brackets_matches_final.dedup();
    closing_brackets_matches_final.sort();
    value_start.dedup();
    value_start.sort();

    let mut concatenated_value: Vec<u8> = Vec::with_capacity(sanitised_bytes.len());
    let mut locale_hash_table: HashTable<TableEntry> = HashTable::new();
    let mut offset = 0;
    for i in 0..opening_brackets_matches_final
        .len()
        .min(closing_brackets_matches_final.len())
        .min(value_start.len())
    {
        let key = std::str::from_utf8(
            &sanitised_bytes
                [opening_brackets_matches_final[i] + 2..closing_brackets_matches_final[i]],
        )
        .expect("Invalid UTF-8 input")
        .trim()
        .as_bytes();
        let value = std::str::from_utf8(
            &sanitised_bytes[value_start[i]
                ..*opening_brackets_matches_final
                    .get(i + 1)
                    .unwrap_or(&sanitised_bytes.len())],
        )
        .expect("Invalid UTF-8 input")
        .trim()
        .as_bytes();
        concatenated_value.extend_from_slice(value);
        insert_into_hashtable(&mut locale_hash_table, key, offset, value.len());
        offset += value.len();
    }
    concatenated_value.shrink_to_fit();

    Ok(LocaleTable {
        unified_box: concatenated_value.into_boxed_slice(),
        entries: locale_hash_table,
    })
}

pub fn insert_into_hashtable(
    table: &mut HashTable<TableEntry>,
    key: &[u8],
    offset: usize,
    length: usize,
) {
    let hash = xxh3_64(&key);
    table.insert_unique(
        hash,
        TableEntry {
            key: hash,
            offset,
            length,
        },
        move |e: &TableEntry| e.key,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_find_entry() {
        let sample =
            b"[[example_key]]hiiii\nexample_value\n##BYEE\n[[another_key]]\nanother_value\n";
        let table = parse_r3locale_bytes(sample).expect("Parse failed");

        let val = table.find_entry(b"example_key");
        assert_eq!(val, Some("example_value"));

        let val2 = table.find_entry(b"another_key");
        assert_eq!(val2, Some("another_value"));

        let missing = table.find_entry(b"missing");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_invalid_utf8() {
        let sample = b"[[bad_key]]\n\xFF\xFE\xFD\n";
        let result = parse_r3locale_bytes(sample);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_value_mismatch() {
        let sample = b"[[only_key]]"; // no value
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::KeyValueMismatch)));
    }

    #[test]
    fn test_bracket_mismatch() {
        let sample = b"[[no_close\nvalue here\n";
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::BracketMismatch)));
    }
}
