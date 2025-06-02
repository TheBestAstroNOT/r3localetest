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
         Some(p) => fs::read(p).expect("Unable to locate locale file"),
         None => Vec::from(include_bytes!("../../src/example.r3l") as &[u8]),
     };
    let mut opening_matches = Vec::new();
    let mut closing_matches = Vec::new();
    let mut i = 0;
    let bytes_len = bytes.len();
    let simd_end = bytes_len.saturating_sub(2 + LANES);
    let mut last_close = 0;
    let mut last_key: Option<Vec<u8>> = None;
    let mut offset: u32 = 0;
    let mut concatenated_value:Vec<Vec<u8>> = Vec::new();

    //Simd search for keys
    while i <= simd_end {
        let opening_bitmask = get_bitmask(i, &bytes, [Some(b'['), Some(b'['), None]);
        for lane in 0..LANES {
            if (opening_bitmask & (1 << lane)) != 0 && ( i + lane == 0 || bytes[i + lane - 1] == b'\n'){
                opening_matches.push(i + lane);
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
        if bytes[i] == b'[' && i >= 1 && bytes[i - 1] == b'[' && bytes[i - 2] == b'\n'{
            opening_matches.push(i - 1);
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


    let mut locale_hashtable: HashTable<TableEntry> = HashTable::with_capacity(opening_matches.len());

    //Parsing values and keys, then adding them to a HashMap
    for (&open_pos, &close_pos) in opening_matches.iter().zip(closing_matches.iter()) {
        if (open_pos < close_pos && close_pos + 2 <= bytes_len) {
            if last_close != 0{
                if let Some(ref key) = last_key {
                let value_slice:&[u8] = &bytes[last_close..open_pos];
                if str::from_utf8(value_slice).is_ok(){
                    let value = std::str::from_utf8(value_slice).expect("If this error message appeared, something is seriously broken!").replace("\r\n", "\n").replace('\r', "\n").trim().as_bytes().to_vec();
                    let value_length = value.len() as u32;
                                    insert(&mut locale_hashtable, key.to_owned(), offset, value_length);
                                    offset += value_length;
                                    concatenated_value.push(value);
                }
                else{
                    panic!("Invalid UTF-8 characters in value");
                }
            }
        }
            let key_slice = &bytes[open_pos + 2..close_pos];
            if str::from_utf8(key_slice).is_ok(){
                let key = std::str::from_utf8(key_slice).expect("If this error message appeared, something is seriously broken!").replace("\r\n", "\n").replace('\r', "\n").trim().as_bytes().to_owned();
                last_key = Some(key);
            }
            last_close = close_pos + 2;
        } else {
            panic!("Skipping invalid range: open={open_pos}, close={close_pos}, bytes_len={bytes_len}");
        }
    }

    // Insert the final value after the last closing bracket (which can be at EOF)
    if let Some(ref key) = last_key {
        if last_close <= bytes_len {
            let value_slice:&[u8] = &bytes[last_close..];
            if str::from_utf8(value_slice).is_ok(){
                let value = std::str::from_utf8(value_slice).expect("If this error message appeared, something is seriously broken!").replace("\r\n", "\n").replace('\r', "\n").trim().as_bytes().to_vec();
                let value_length = value.len() as u32;
                                insert(&mut locale_hashtable, key.to_owned(), offset, value_length);
                                offset += value_length;
                                concatenated_value.push(value);
            }
            else{
                panic!("Invalid UTF-8 characters in value");
            }
        } else {
            let value = String::new().as_bytes().to_vec();
            let value_length = value.len() as u32;
            insert(&mut locale_hashtable, key.to_owned(), offset, value_length);
            offset += value_length;
            concatenated_value.push(value);
        }
    }

    //Returning it
    LocaleTable{unified_box: build_boxed_buffer(concatenated_value, offset), entries: locale_hashtable}
}

pub fn insert(table: &mut HashTable<TableEntry>, key: Vec<u8>, offset: u32, length: u32){
    let hash = xxh3_64(&key);
    table.insert_unique(hash, TableEntry{key: hash, offset, length}, move |e: &TableEntry| { e.key });
}

fn build_boxed_buffer(parts: Vec<Vec<u8>>, length: u32) -> Box<[u8]> {
    let mut buffer_uninit = Box::<[u8]>::new_uninit_slice(length as usize);
    let mut offset = 0;
    let ptr = buffer_uninit.as_mut_ptr() as *mut u8;
    for part in parts {
        let bytes = part;
        unsafe {
            // Copy bytes into uninitialized buffer
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.add(offset), bytes.len());
        }
        offset += bytes.len();
    }

    unsafe { buffer_uninit.assume_init() }
}