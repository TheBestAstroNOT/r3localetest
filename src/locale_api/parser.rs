use super::interop::LocaleTable;
use super::interop::TableEntry;
use super::sanitizer::sanitize_r3_locale_file;
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
        if insert_into_hashtable(&mut locale_hash_table, key, offset, value.len()).is_err() {
            return Err(ParseR3Error::DuplicateKeys);
        }

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
) -> Result<(), ParseR3Error> {
    let hash = xxh3_64(key);
    if table
        .find(hash, |table_entry: &TableEntry| table_entry.key == hash)
        .is_none()
    {
        table.insert_unique(
            hash,
            TableEntry {
                key: hash,
                offset,
                length,
            },
            move |e: &TableEntry| e.key,
        );
        Ok(())
    } else {
        Err(ParseR3Error::DuplicateKeys)
    }
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
        assert!(matches!(result, Err(ParseR3Error::InvalidUTF8Value)));
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

    #[test]
    fn test_duplicate_keys() {
        let sample = b"[[duplicate_key]]\nfirst_value\n[[duplicate_key]]\nsecond_value";
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::DuplicateKeys)));
    }
}

#[repr(C)]
pub struct MergeResult {
    pub table: *mut LocaleTable,
    pub merge_state: MergeTableError,
}

pub fn get_locale_table_rust(path: &Path) -> Result<LocaleTable, ParseR3Error> {
    parse_r3locale_file(path)
}

pub fn merge_locale_table_rust(tables: &[&LocaleTable]) -> MergeResult {
    let initial_hasher = |entry: &(TableEntry, &Box<[u8]>)| entry.0.key;
    let final_hasher = |entry: &TableEntry| entry.key;
    let mut initial_table: HashTable<(TableEntry, &Box<[u8]>)> = HashTable::new();

    for table in tables {
        for entry in table.entries.iter() {
            if initial_table
                .find(entry.key, |table_entry: &(TableEntry, &Box<[u8]>)| {
                    table_entry.0.key == entry.key
                })
                .is_none()
            {
                initial_table.insert_unique(
                    entry.key,
                    (*entry, &table.unified_box),
                    initial_hasher,
                );
            }
        }
    }

    let mut final_table: HashTable<TableEntry> = HashTable::new();
    let mut final_buffer: Vec<u8> = Vec::new();
    for entry in initial_table.iter() {
        final_table.insert_unique(
            entry.0.key,
            TableEntry {
                key: entry.0.key,
                length: entry.0.length,
                offset: final_buffer.len(),
            },
            final_hasher,
        );
        final_buffer.extend_from_slice(&entry.1[entry.0.offset..entry.0.offset + entry.0.length]);
    }

    let final_boxed_buffer = final_buffer.into_boxed_slice();
    MergeResult {
        table: Box::into_raw(Box::new(LocaleTable {
            unified_box: final_boxed_buffer,
            entries: final_table,
        })),
        merge_state: MergeTableError::Normal,
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum ParseR3Error {
    Normal,
    FileNotFound,
    FailedToRead,
    KeyValueMismatch,
    BracketMismatch,
    InvalidUTF8Value,
    InvalidUTF8Path,
    NullPathProvided,
    DuplicateKeys,
}

#[derive(Debug)]
#[repr(C)]
pub enum MergeTableError {
    Normal,
    NullTablePointer,
    FileNotFound,
    FailedToRead,
    KeyValueMismatch,
    BracketMismatch,
    InvalidUTF8Value,
    InvalidUTF8Path,
    NullPathProvided,
    DuplicateKeys,
}

impl From<ParseR3Error> for MergeTableError {
    fn from(err: ParseR3Error) -> Self {
        match err {
            ParseR3Error::Normal => MergeTableError::Normal,
            ParseR3Error::FileNotFound => MergeTableError::FileNotFound,
            ParseR3Error::FailedToRead => MergeTableError::FailedToRead,
            ParseR3Error::KeyValueMismatch => MergeTableError::KeyValueMismatch,
            ParseR3Error::BracketMismatch => MergeTableError::BracketMismatch,
            ParseR3Error::InvalidUTF8Value => MergeTableError::InvalidUTF8Value,
            ParseR3Error::InvalidUTF8Path => MergeTableError::InvalidUTF8Path,
            ParseR3Error::NullPathProvided => MergeTableError::NullPathProvided,
            ParseR3Error::DuplicateKeys => MergeTableError::DuplicateKeys,
        }
    }
}
