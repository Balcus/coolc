use crate::{
    semantic_analysis::{inheritance_tree::InheritanceTree, symbol_table::SymbolTable},
    string_table::StringTable,
};

type SymbolId = usize;
type TypeId = usize;
type ClassId = usize;

pub mod inheritance_tree;
pub mod symbol_table;

#[derive(Debug, PartialEq)]
pub enum SemanticError {
    InheritanceCycle,
    DuplicateClass,
    NonExistentClass(usize),
}

pub enum VarKind {
    Local,
    Formal,
    Attribute,
    SelfObject,
}

pub struct VarInfo {
    ty: TypeId,
    kind: VarKind,
}

pub struct MethodInfo {
    ret_ty: TypeId,
    formal_info: Vec<FormalInfo>,
}

pub struct FormalInfo {
    ty: TypeId,
}

pub struct Environment<'a> {
    string_table: &'a StringTable,
    obejct_env: &'a mut SymbolTable<SymbolId, VarInfo>,
    inheritance_tree: &'a InheritanceTree,
    current_class: ClassId,
}
