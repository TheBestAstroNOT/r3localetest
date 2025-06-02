use hashbrown::HashTable;
use xxhash_rust::xxh3::xxh3_64;

pub struct LocaleTable {
    pub unified_box: Box<[u8]>,
    pub entries: HashTable<TableEntry>,
}

pub struct TableEntry {
    pub key: u64,
    pub offset: usize,
    pub length: usize,
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

    pub fn find_entry(&self, key: &[u8]) -> Option<&str> {
        let hash = xxh3_64(key);
        println!("{}", hash);
        self.entries
            .find(hash, |entry| entry.key == hash)
            .and_then(|entry| {
                let slice = &self.unified_box[entry.offset..entry.offset + entry.length];
                str::from_utf8(slice).ok()
            })
    }
}
