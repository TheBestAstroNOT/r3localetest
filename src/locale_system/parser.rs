use std::fs;
use std::path::Path;
use hashbrown::HashTable;
use super::simd::{get_bitmask, LANES};
use super::types::LocaleTable;
use super::types::TableEntry;
use xxhash_rust::xxh3::xxh3_64;

//Parses a reloaded 3 localisation file and returns a LocaleTable
pub fn parse_r3locale_file(path: Option<&Path>) -> LocaleTable{
    //Initialising all variables
     let bytes: Vec<u8> = match path {
         Some(p) => fs::read(p).expect(&format!("Unable to locate locale file: {}", p.display())),
         None => Vec::from(include_bytes!("../../src/bigexample.r3l") as &[u8]),
     };
    let mut opening_matches = Vec::new();
    let mut closing_matches = Vec::new();
    let mut i = 0;
    let bytes_len = bytes.len();
    let simd_end = bytes_len.saturating_sub(2 + LANES);
    let mut last_close = 0;
    let mut last_key: Option<u64> = None;
    let mut offset: u32 = 0;
    let mut concatenated_value:Vec<String> = Vec::new();

    //Simd search for keys
    while i <= simd_end {
        let opening_bitmask = get_bitmask(i, &bytes, [Some(b'['), Some(b'['), None]);
        for lane in 0..LANES {
            if (opening_bitmask & (1 << lane)) != 0 {
                if i + lane == 0 || bytes[i + lane - 1] == b'\n' {
                    opening_matches.push(i + lane);
                }
            }
        }

        let closing_bitmask = get_bitmask(i, &bytes, [Some(b']'), Some(b']'), None]);
        for lane in 0..LANES {
            if (closing_bitmask & (1 << lane)) != 0 {
                closing_matches.push(i + lane);
            }
        }

        i += LANES;
    }

    //Scalar search for keys
    while i < bytes_len {
        if bytes[i] == b'[' && i >= 1 && bytes[i - 1] == b'[' {
            if bytes[i - 2] == b'\n' {
                opening_matches.push(i - 1);
            }
        }

        if bytes[i] == b']' && i >= 1 && bytes[i - 1] == b']' {
            closing_matches.push(i - 1);
        }

        i += 1;
    }

    //Cleaning up collected indices
    opening_matches.sort_unstable();
    opening_matches.dedup();
    closing_matches.sort_unstable();
    closing_matches.dedup();

    //Safety check for matches
    if opening_matches.len() != closing_matches.len() {
        #[cfg(feature = "additional_locale_safety_checks")]
        {
            panic!("Opening and Closing bracket mismatch! Openings: {}, Closings: {}", opening_matches.len(), closing_matches.len());
        }
    }


    let locale_hashtable: HashTable<TableEntry> = HashTable::with_capacity(opening_matches.len());
    let mut locale_map: LocaleTable = LocaleTable{unified_address: None, entries: locale_hashtable};

    //Parsing values and keys, then adding them to a HashMap
    for (&open_pos, &close_pos) in opening_matches.iter().zip(closing_matches.iter()) {
        if open_pos < close_pos && close_pos + 2 <= bytes_len {
            if last_close != 0 {
                if let Some(ref key) = last_key {
                    let value = normalize_newlines(
                        String::from_utf8_lossy(&bytes[last_close..open_pos])
                            .trim()
                    );
                    let value_length = value.as_bytes().len() as u32;
                    locale_map.insert(TableEntry {
                                        key: *key,
                                        offset,
                                        length: value_length,
                                    });
                    offset += value_length;
                    concatenated_value.push(value);
                }
            }

            let key = xxh3_64(normalize_newlines(
                    String::from_utf8_lossy(&bytes[open_pos + 2..close_pos])
                    .trim()
                ).as_bytes()
            );
            last_key = Some(key);
            last_close = close_pos + 2;
        } else {
            panic!(
                "Skipping invalid range: open={}, close={}, bytes_len={}",
                open_pos, close_pos, bytes_len
            );
        }
    }

    // Insert the final value after the last closing bracket (which can be at EOF)
    if let Some(ref key) = last_key {
        if last_close <= bytes_len {
            let value = normalize_newlines(
                String::from_utf8_lossy(&bytes[last_close..])
                    .trim()
            );
            let value_length = value.len() as u32;
            locale_map.insert(TableEntry {
                                            key: *key,
                                            offset,
                                            length: value_length,
                                        });
            offset += value_length;
            concatenated_value.push(value);
        } else {
            let value = String::new();
            let value_length = value.len() as u32;
            locale_map.insert(TableEntry {
                                                    key: *key,
                                                    offset,
                                                    length: value_length,
                                                });
            offset += value_length;
            concatenated_value.push(value);
        }
    }

    let buffer_uninit = build_boxed_buffer(concatenated_value, offset);
    locale_map.unified_address = Some(buffer_uninit.as_ptr());

    //Returning it
    locale_map
}

fn build_boxed_buffer(parts: Vec<String>, length: u32) -> Box<[u8]> {
    let mut buffer_uninit = Box::<[u8]>::new_uninit_slice(length as usize);
    let mut offset = 0;
    let ptr = buffer_uninit.as_mut_ptr() as *mut u8;
    for part in parts {
        let bytes = part.as_bytes();
        unsafe {
            // Copy bytes into uninitialized buffer
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.add(offset), bytes.len());
        }
        offset += bytes.len();
    }

    unsafe { buffer_uninit.assume_init() }
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}