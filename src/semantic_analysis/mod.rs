pub mod inheritance_tree;

#[derive(Debug, PartialEq)]
pub enum SemanticError {
    InheritanceCycle,
    DuplicateClass,
    NonExistentClass(usize),
}