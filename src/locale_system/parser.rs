use std::fs;
use memchr::{memchr, memmem};
use std::path::Path;
use hashbrown::HashTable;
use super::types::LocaleTable;
use super::types::TableEntry;
use xxhash_rust::xxh3::xxh3_64;
use super::sanitizer::sanitize_r3_locale_file;

//Parses a reloaded 3 localisation file and returns a LocaleTable
pub fn parse_r3locale_file(path: Option<&Path>) -> LocaleTable {
    let bytes: Vec<u8> = match path {
             Some(p) => fs::read(p).expect("Unable to locate locale file"),
             None => Vec::from(include_bytes!("../../src/example.r3l") as &[u8]),
     };
    let sanitised_bytes = sanitize_r3_locale_file(&*bytes);
    let opening_brackets_matches_initial:Vec<usize> = memmem::find_iter(&sanitised_bytes, b"[[").collect();
    let mut opening_brackets_matches_final:Vec<usize> = Vec::with_capacity(opening_brackets_matches_initial.len());
    let mut closing_brackets_matches_final:Vec<usize> = Vec::with_capacity(opening_brackets_matches_initial.len());
    let mut value_start:Vec<usize> = Vec::with_capacity(opening_brackets_matches_initial.len());
    for item in &opening_brackets_matches_initial{
        if *item == 0 || sanitised_bytes[item - 1] == b'\n'{
            opening_brackets_matches_final.push(*item);
            if let Some(close_pos) = memmem::find(&sanitised_bytes[*item..], b"]]") {
                closing_brackets_matches_final.push(item + close_pos);
                if let Some(value_open_pos) = memchr(b'\n', &sanitised_bytes[item + close_pos..]) {
                    value_start.push(item + close_pos + value_open_pos);
                }
                else{
                    panic!("No value found for key!")
                }
            } else {
                panic!("No closing bracket found!");
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
    for i in 0..opening_brackets_matches_final.len().min(closing_brackets_matches_final.len()).min(value_start.len()) {
        let key = std::str::from_utf8(&sanitised_bytes[opening_brackets_matches_final[i]+2..closing_brackets_matches_final[i]]).expect("Invalid UTF-8 input").trim().as_bytes();
        let value = std::str::from_utf8(&sanitised_bytes[value_start[i]..*opening_brackets_matches_final.get(i + 1).unwrap_or(&sanitised_bytes.len())]).expect("Invalid UTF-8 input").trim().as_bytes();
        concatenated_value.extend_from_slice(value);
        insert_into_hashtable(&mut locale_hash_table, key, offset, value.len());
        offset += value.len();
    }
    concatenated_value.shrink_to_fit();

    LocaleTable{unified_box: concatenated_value.into_boxed_slice(), entries: locale_hash_table}
}

pub fn insert_into_hashtable(table: &mut HashTable<TableEntry>, key: &[u8], offset: usize, length: usize){
    let hash = xxh3_64(&key);
    table.insert_unique(hash, TableEntry{key: hash, offset, length}, move |e: &TableEntry| { e.key });
}

#[test]
fn run(){
    let table = parse_r3locale_file(None);
    table.show_all_entries();
}