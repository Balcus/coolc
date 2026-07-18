pub mod inheritance_tree;
pub mod symbol_table;

#[derive(Debug, PartialEq)]
pub enum SemanticError {
    InheritanceCycle,
    DuplicateClass,
    NonExistentClass(usize),
}