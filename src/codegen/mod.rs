use std::format;

use crate::{
    codegen::{
        class_info::ClassInfo,
        constants::{BOOL_TAG_LABEL, INT_TAG_LABEL, STRING_TAG_LABEL},
        emitter::emit_globals,
        gc::GcMode,
    },
    semantic_analysis::{ast, method_table::MethodTable},
    string_table::StringTable,
    utils::TagGen,
};

mod class_info;
mod constants;
mod emitter;
pub mod gc;

pub struct CodeGen {
    tag_gen: TagGen,
    class_info: Vec<ClassInfo>,
    s_table: StringTable,
    method_table: MethodTable,
    gc_mode: GcMode,
}

impl CodeGen {
    // TODO: add attributes and methods for the builtins later
    pub fn new(s_table: StringTable, method_table: MethodTable, gc_mode: GcMode) -> Self {
        let mut tag_gen = TagGen::default();

        let class_info: Vec<ClassInfo> = vec![
            ClassInfo::from(
                s_table.lookup("Object").unwrap(),
                String::from("Object"),
                tag_gen.get(),
                None,
                Vec::new(),
                Vec::new(),
            ),
            ClassInfo::from(
                s_table.lookup("IO").unwrap(),
                String::from("IO"),
                tag_gen.get(),
                Some(s_table.lookup("Object").unwrap()),
                Vec::new(),
                Vec::new(),
            ),
            ClassInfo::from(
                s_table.lookup("Int").unwrap(),
                String::from("Int"),
                tag_gen.get(),
                Some(s_table.lookup("Object").unwrap()),
                Vec::new(),
                Vec::new(),
            ),
            ClassInfo::from(
                s_table.lookup("Bool").unwrap(),
                String::from("Bool"),
                tag_gen.get(),
                Some(s_table.lookup("Object").unwrap()),
                Vec::new(),
                Vec::new(),
            ),
            ClassInfo::from(
                s_table.lookup("String").unwrap(),
                String::from("String"),
                tag_gen.get(),
                Some(s_table.lookup("Object").unwrap()),
                Vec::new(),
                Vec::new(),
            ),
        ];

        Self {
            tag_gen,
            class_info,
            s_table,
            method_table,
            gc_mode,
        }
    }

    pub fn add_class(&mut self, class: ClassInfo) {
        self.class_info.push(class);
    }

    fn tag_of(&self, class_name: &str) -> usize {
        self.class_info
            .iter()
            .find(|class| class.name() == "Int")
            .unwrap()
            .tag()
    }

    fn emit_tag_builtins(&self) -> String {
        format!(
            concat!(
                "{int_label}:\n",
                "        .word   {int}\n",
                "{bool_label}:\n",
                "        .word   {bool_}\n",
                "{string_label}:\n",
                "        .word   {string}\n",
            ),
            int_label = INT_TAG_LABEL,
            bool_label = BOOL_TAG_LABEL,
            string_label = STRING_TAG_LABEL,
            int = self.tag_of("Int"),
            bool_ = self.tag_of("Bool"),
            string = self.tag_of("String"),
        )
    }

    pub fn codegen(&mut self, from: ast::Root) -> String {
        let mut res = String::with_capacity(64 * 1024);

        // Data section
        res.push_str(".data\n");
        res.push_str(&emit_globals());
        res.push_str(&self.emit_tag_builtins());

        res
    }
}
