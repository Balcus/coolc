
use crate::{codegen::emitter::emit_globals, inheritance_tree::InheritanceTree, semantic_analysis::{ast, method_table::MethodTable}};

mod constants;
mod emitter;
mod class_info;

pub fn codegen(root: ast::Root, inheritance_tree: &InheritanceTree, method_table: &MethodTable) -> String {
    let mut res = String::with_capacity(64 * 1024);

    if let Err(e) = emit_globals(&mut res) {
        eprintln!("[Codegen] Failed to write globals into output String. {}", e);
    }

    res.push_str(".data");

    // TODO:
    // class prototypes
    // method dispatch tables
    // constants as objects of type Int, Bool, String, Object



    res.push_str(".text");

    res
}