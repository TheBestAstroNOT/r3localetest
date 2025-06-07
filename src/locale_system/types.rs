use crate::locale_system::parse_r3locale_file;
use crate::locale_system::parser::ParseR3Error;
use hashbrown::HashTable;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

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
pub extern "C" fn get_locale_table(path: *const c_char) -> AllocationResult {
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

    match parse_r3locale_file(Some(Path::new(path_str))) {
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
pub extern "C" fn locale_table_find_utf8(
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
        return FindEntryResult {
            value_ptr: std::ptr::null(),
            value_len: 0,
            allocation_state: FindEntryError::NoEntryFound,
        };
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

#[unsafe(no_mangle)]
pub extern "C" fn free_locale_table(ptr: *mut LocaleTable) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

impl LocaleTable {
    pub fn show_all_entries(&self) {
        for entry in self.entries.iter() {
            let data_slice = &self.unified_box[entry.offset..entry.offset + entry.length];
            match str::from_utf8(data_slice) {
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
            .and_then(|entry| return Some((entry.offset, entry.length)));
        None
    }

    pub fn find_entry(&self, key: &[u8]) -> Option<&str> {
        let hash = xxh3_64(key);
        self.entries
            .find(hash, |entry| entry.key == hash)
            .and_then(|entry| {
                let slice = &self.unified_box[entry.offset..entry.offset + entry.length];
                str::from_utf8(slice).ok()
            })
    }
}
