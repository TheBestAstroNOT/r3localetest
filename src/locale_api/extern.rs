use super::parser::{MergeResult, MergeTableError, ParseR3Error, parse_r3locale_file};
use crate::locale_api::parser;
use hashbrown::HashTable;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TableEntry {
    pub key: u64,
    pub offset: usize,
    pub length: usize,
}

#[repr(C)]
pub struct LocaleTable {
    pub unified_box: Box<[u8]>,
    pub entries: HashTable<TableEntry>,
}

#[repr(C)]
pub struct AllocationResult {
    pub table: *mut LocaleTable,
    pub allocation_state: ParseR3Error,
}

#[repr(C)]
pub struct FindEntryResult {
    pub value_ptr: *const u8,
    pub value_len: usize,
    pub allocation_state: FindEntryError,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn merge_locale_table_c(
    tables: *const *const LocaleTable,
    count: usize,
) -> MergeResult {
    //NOTE: DO NOT FORGET TO NOTE THAT THE FIRST ITEM IN THE ARRAY OF POINTERS WILL WIN

    if tables.is_null() {
        return MergeResult {
            table: std::ptr::null_mut(),
            merge_state: MergeTableError::NullTablePointer,
        };
    }

    parser::merge_locale_table_rust(unsafe {
        std::slice::from_raw_parts(tables as *const &LocaleTable, count)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_locale_table(path: *const c_char) -> AllocationResult {
    if path.is_null() {
        return AllocationResult {
            table: std::ptr::null_mut(),
            allocation_state: ParseR3Error::NullPathProvided,
        };
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            return AllocationResult {
                table: std::ptr::null_mut(),
                allocation_state: ParseR3Error::InvalidUTF8Path,
            };
        }
    };

    match parse_r3locale_file(Path::new(path_str)) {
        Ok(table) => AllocationResult {
            table: Box::into_raw(Box::new(table)),
            allocation_state: ParseR3Error::Normal,
        },
        Err(parse_error) => AllocationResult {
            table: std::ptr::null_mut(),
            allocation_state: parse_error,
        },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_multiple_locale_tables(
    paths: *const *const c_char,
    count: usize,
) -> MergeResult {
    if paths.is_null() {
        return MergeResult {
            table: std::ptr::null_mut(),
            merge_state: MergeTableError::NullPathProvided,
        };
    }

    // Convert raw pointer to slice
    let path_slice = unsafe { std::slice::from_raw_parts(paths, count) };

    let mut parsed_tables = Vec::with_capacity(count);
    for &c_path in path_slice {
        if c_path.is_null() {
            return MergeResult {
                table: std::ptr::null_mut(),
                merge_state: MergeTableError::NullPathProvided,
            };
        }

        let c_str = unsafe { CStr::from_ptr(c_path) };
        let path_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => {
                return MergeResult {
                    table: std::ptr::null_mut(),
                    merge_state: MergeTableError::InvalidUTF8Path,
                };
            }
        };

        match parse_r3locale_file(Path::new(path_str)) {
            Ok(table) => parsed_tables.push(table),
            Err(parse_error) => {
                return MergeResult {
                    table: std::ptr::null_mut(),
                    merge_state: parse_error.into(),
                };
            }
        }
    }

    // References to all tables for merging
    let references: Vec<&LocaleTable> = parsed_tables.iter().collect();
    parser::merge_locale_table_rust(&references)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_entry(
    table: *const LocaleTable,
    key_ptr: *const u8,
    key_len: usize,
) -> FindEntryResult {
    if table.is_null() {
        return FindEntryResult {
            value_ptr: std::ptr::null(),
            value_len: 0,
            allocation_state: FindEntryError::NullTable,
        };
    } else if key_ptr.is_null() {
        return FindEntryResult {
            value_ptr: std::ptr::null(),
            value_len: 0,
            allocation_state: FindEntryError::NullKeyPtr,
        };
    }

    let table = unsafe { &*table };
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };

    if let Some((offset, length)) = table.find_entry_raw(key) {
        unsafe {
            FindEntryResult {
                value_ptr: table.unified_box.as_ptr().add(offset),
                value_len: length,
                allocation_state: FindEntryError::Normal,
            }
        }
    } else {
        FindEntryResult {
            value_ptr: std::ptr::null(),
            value_len: 0,
            allocation_state: FindEntryError::NoEntryFound,
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_locale_table(ptr: *mut LocaleTable) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

impl LocaleTable {
    pub fn show_all_entries(&self) {
        for entry in self.entries.iter() {
            let data_slice = &self.unified_box[entry.offset..entry.offset + entry.length];
            match std::str::from_utf8(data_slice) {
                Ok(value_str) => {
                    println!("Key: {:016x}, Value: {}", entry.key, value_str);
                }
                Err(_) => {
                    println!("Key: {:016x}, Value: <invalid UTF-8>", entry.key);
                }
            }
        }
    }

    pub fn find_entry_raw(&self, key: &[u8]) -> Option<(usize, usize)> {
        let hash = xxh3_64(key);
        self.entries
            .find(hash, |entry| entry.key == hash)
            .map(|entry| return (entry.offset, entry.length));
        None
    }

    pub fn find_entry(&self, key: &[u8]) -> Option<&str> {
        let hash = xxh3_64(key);
        self.entries
            .find(hash, |entry| entry.key == hash)
            .and_then(|entry| {
                let slice = &self.unified_box[entry.offset..entry.offset + entry.length];
                std::str::from_utf8(slice).ok()
            })
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum FindEntryError {
    Normal,
    NullTable,
    NullKeyPtr,
    NoEntryFound,
}
