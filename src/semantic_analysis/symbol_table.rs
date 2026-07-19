use std::collections::HashMap;
use std::hash::Hash;

// The symbol table is implemented as a stack of scopes
//
// Each scope is represented by a HashMap mapping a symbol identifier
// (in our case usize) to a SymbolEntry which stores information
// associated with that symbol.
//
// Entering a scope pushes a new, initially empty, HashMap onto the stack
// Exiting a scope pops the top HashMap from the stack
//
// Adding a symbol inserts a new (symbol_id -> SymbolEntry) mapping into
// the current scope, the HashMap that sits at the top of the stack.
//
// Looking up a symbol searches the scopes from the innermost (top of the
// stack) to the outermost (bottom of the stack), returning the first match.
#[derive(Debug)]
pub struct SymbolTable<SYM, DAT> {
    scopes: Vec<HashMap<SYM, DAT>>,
}

impl<SYM: Eq + Hash, DAT> SymbolTable<SYM, DAT> {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes
            .pop()
            .expect("Impossible to pop scopes while no scope exists");
    }

    pub fn add_id(&mut self, symbol_id: SYM, data: DAT) {
        let last = self.scopes.last_mut().expect(
            "There needs to be at least one scope in the symbol table for the add operation",
        );
        last.insert(symbol_id, data);
    }

    pub fn lookup(&self, symbol_id: &SYM) -> Option<&DAT> {
        for scope in self.scopes.iter().rev() {
            if let Some(data) = scope.get(symbol_id) {
                return Some(data);
            }
        }
        None
    }

    pub fn probe(&self, symbol_id: &SYM) -> Option<&DAT> {
        self.scopes
            .last()
            .expect(
                "There needs to be at least one scope in the symbol table for the probe operation",
            )
            .get(symbol_id)
    }

    pub fn scopes_len(&self) -> usize {
        self.scopes.len()
    }
}
