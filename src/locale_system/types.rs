use xxhash_rust::xxh3::xxh3_64;
use hashbrown::HashTable;

pub struct LocaleTable {
    pub unified_address: Option<*const u8>,
    pub entries: HashTable<TableEntry>
}

pub struct TableEntry
{
    pub key: u64,
    pub offset: u32,
    pub length: u32,
}

impl LocaleTable{
    pub fn insert(&mut self, entry: TableEntry){
        self.entries.insert_unique(xxh3_64(&entry.key.to_le_bytes()), entry, move |e: &TableEntry| {
            xxh3_64(&e.key.to_le_bytes())
        });
    }
}
