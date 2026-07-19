use std::collections::HashMap;

pub const OBJECT_ID: usize = 0;
pub const INT_ID: usize = 1;
pub const BOOL_ID: usize = 2;
pub const STRING_ID: usize = 3;

pub struct StringTable {
    pub map: HashMap<String, usize>,
}

impl StringTable {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("Object".to_string(), OBJECT_ID);
        map.insert("Int".to_string(), INT_ID);
        map.insert("Bool".to_string(), BOOL_ID);
        map.insert("String".to_string(), STRING_ID);
        Self { map }
    }

    pub fn insert(&mut self, string: String) -> usize {
        let next_id = self.map.len();
        *self.map.entry(string).or_insert(next_id)
    }

    pub fn lookup(&self, string: &str) -> Option<usize> {
        self.map.get(string).copied()
    }

    pub fn string_from_id(&self, id: usize) -> Option<&String> {
        self.map.iter().find(|(_, v)| *v == &id).map(|(k, _)| k)
    }
}
