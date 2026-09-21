use crate::semantic_analysis::builtins::{
    ABORT_ID, BOOL_ID, CONCAT_ID, COPY_ID, I_ID, IN_INT_ID, IN_STRING_ID, INT_ID, IO_ID, L_ID,
    LENGTH_ID, OBJECT_ID, OUT_INT_ID, OUT_STRING_ID, S_ID, STRING_ID, SUBSTR_ID, TYPE_NAME_ID,
    X_ID,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct StringTable {
    map: HashMap<String, usize>,
}

impl StringTable {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        map.insert("Object".to_string(), OBJECT_ID);
        map.insert("Int".to_string(), INT_ID);
        map.insert("Bool".to_string(), BOOL_ID);
        map.insert("String".to_string(), STRING_ID);
        map.insert("IO".to_string(), IO_ID);

        map.insert("abort".to_string(), ABORT_ID);
        map.insert("type_name".to_string(), TYPE_NAME_ID);
        map.insert("copy".to_string(), COPY_ID);
        map.insert("out_string".to_string(), OUT_STRING_ID);
        map.insert("out_int".to_string(), OUT_INT_ID);
        map.insert("in_string".to_string(), IN_STRING_ID);
        map.insert("in_int".to_string(), IN_INT_ID);
        map.insert("length".to_string(), LENGTH_ID);
        map.insert("concat".to_string(), CONCAT_ID);
        map.insert("substr".to_string(), SUBSTR_ID);

        map.insert("x".to_string(), X_ID);
        map.insert("i".to_string(), I_ID);
        map.insert("l".to_string(), L_ID);
        map.insert("s".to_string(), S_ID);

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
