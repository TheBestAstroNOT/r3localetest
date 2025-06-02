use hashbrown::HashTable;

pub struct LocaleTable {
    pub unified_box: Box<[u8]>,
    pub entries: HashTable<TableEntry>
}

pub struct TableEntry
{
    pub key: u64,
    pub offset: u32,
    pub length: u32,
}
