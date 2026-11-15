use crate::builtins::{
    ABORT_ID, BOOL_ID, CONCAT_ID, COPY_ID, I_ID, IN_INT_ID, IN_STRING_ID, INT_ID, IO_ID, L_ID,
    LENGTH_ID, OBJECT_ID, OUT_INT_ID, OUT_STRING_ID, S_ID, STRING_ID, SUBSTR_ID, TYPE_NAME_ID,
    X_ID,
};

use std::collections::HashMap;

#[derive(Debug)]
pub struct StringTable {
    map: HashMap<String, usize>,
    strings: Vec<String>,
}

impl StringTable {
    pub fn new() -> Self {
        let mut table = Self {
            map: HashMap::new(),
            strings: Vec::new(),
        };

        table.insert_at("Object", OBJECT_ID);
        table.insert_at("Int", INT_ID);
        table.insert_at("Bool", BOOL_ID);
        table.insert_at("String", STRING_ID);
        table.insert_at("IO", IO_ID);

        table.insert_at("abort", ABORT_ID);
        table.insert_at("type_name", TYPE_NAME_ID);
        table.insert_at("copy", COPY_ID);
        table.insert_at("out_string", OUT_STRING_ID);
        table.insert_at("out_int", OUT_INT_ID);
        table.insert_at("in_string", IN_STRING_ID);
        table.insert_at("in_int", IN_INT_ID);
        table.insert_at("length", LENGTH_ID);
        table.insert_at("concat", CONCAT_ID);
        table.insert_at("substr", SUBSTR_ID);

        table.insert_at("x", X_ID);
        table.insert_at("i", I_ID);
        table.insert_at("l", L_ID);
        table.insert_at("s", S_ID);

        table.insert(String::new());

        table
    }

    fn insert_at(&mut self, name: &str, id: usize) {
        if self.strings.len() <= id {
            self.strings.resize(id + 1, String::new());
        }
        self.strings[id] = name.to_string();
        self.map.insert(name.to_string(), id);
    }

    pub fn insert(&mut self, string: String) -> usize {
        if let Some(&id) = self.map.get(&string) {
            return id;
        }
        let id = self.strings.len();
        self.strings.push(string.clone());
        self.map.insert(string, id);
        id
    }

    pub fn lookup(&self, string: &str) -> Option<usize> {
        self.map.get(string).copied()
    }

    pub fn string_from_id(&self, id: usize) -> Option<&String> {
        self.strings.get(id)
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }
}
