use crate::{
    codegen::{
        class_info::codegen_class,
        emitter::{emit_globals, emit_proto_builtins},
    },
    semantic_analysis::{ast, method_table::MethodTable},
    string_table::StringTable,
};

mod class_info;
mod constants;
mod emitter;

struct TagGen {
    next: usize,
}

impl TagGen {
    pub fn new() -> Self {
        TagGen { next: 0 }
    }

    pub fn get(&mut self) -> usize {
        let tag = self.next;
        self.next += 1;
        tag
    }
}

pub struct LabelGen {
    next: usize,
}

impl LabelGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }
    pub fn get(&mut self) -> String {
        let l = format!("label{}", self.next);
        self.next += 1;
        l
    }
}

pub fn codegen(root: ast::Root, method_table: &MethodTable, s_table: &StringTable) -> String {
    // preallocate 64 kB
    let mut res = String::with_capacity(64 * 1024);

    res.push_str(&emit_globals());
    res.push_str(".data");

    let mut tag_gen = TagGen::new();
    res.push_str(&emit_proto_builtins(&mut tag_gen));

    for class in root.classes {
        res.push_str(&codegen_class(class, &mut tag_gen, s_table));
    }

    // TODO:
    // class prototypes
    // method dispatch tables
    // constants as objects of type Int, Bool, String, Object

    res.push_str(".text");

    res
}
