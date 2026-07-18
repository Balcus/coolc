use std::collections::HashMap;

pub struct StringTable {
    pub map: HashMap<String, usize>,
}

impl StringTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
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
