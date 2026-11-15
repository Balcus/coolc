use std::collections::HashMap;

use crate::{
    builtins::{BOOL_ID, BUILTINS, INT_ID, STRING_ID},
    codegen::{
        class_info::{AttrInfo, ClassInfo, MethodInfo},
        const_tbl::ConstTable,
        constants::HEADER_WORDS,
        emitter::{
            emit_bool_const, emit_gc, emit_globals, emit_int_const, emit_string_const,
            emit_tags_builtins,
        },
        gc::GcMode,
    },
    inheritance_tree::InheritanceTree,
    semantic_analysis::{ast, method_table::MethodTable},
    string_table::StringTable,
    utils::TagGen,
};

mod class_info;
mod const_tbl;
mod constants;
mod emitter;
pub mod gc;

pub struct CodeGen {
    src: ast::Root,
    tag_gen: TagGen,
    class_info: Vec<ClassInfo>,
    s_table: StringTable,
    method_table: MethodTable,
    gc_mode: GcMode,
    constants: ConstTable,
    tags: HashMap<usize, usize>,
}

impl CodeGen {
    // TODO: add attributes and methods for the builtins later
    pub fn from(
        src: ast::Root,
        s_table: StringTable,
        inh_tree: InheritanceTree,
        method_table: MethodTable,
        gc_mode: GcMode,
    ) -> Self {
        let mut tag_gen = TagGen::default();
        let constants = ConstTable::from(&src, &s_table);

        let mut res = Self {
            src,
            tag_gen,
            class_info: Vec::new(),
            s_table,
            method_table,
            gc_mode,
            constants,
            tags: HashMap::new(),
        };

        for builtin in BUILTINS {
            res.add_class(&ast::ClassNode::from(builtin));
        }

        let mut classes = res.src.classes.clone();
        classes.sort_by_key(|c| inh_tree.ancestors(c.id).len());
        for class in &classes {
            res.add_class(class);
        }

        res
    }

    pub fn add_class(&mut self, class: &ast::ClassNode) {
        let tag = self.tag_gen.get();
        let name = self.s_table.string_from_id(class.id).unwrap().clone();

        let (mut attrs, mut methods) = match class.parent {
            Some(p) => {
                let info = self.class_info_of(p);
                (info.attrs().to_vec(), info.methods().to_vec())
            }
            None => (Vec::new(), Vec::new()),
        };

        for feature in &class.features {
            match feature {
                ast::FeatureNode::Attribute { id, type_dec, .. } => {
                    let offset = 12 + 4 * attrs.len();
                    let attr_name = self.s_table.string_from_id(*id).unwrap().clone();
                    attrs.push(AttrInfo::new(attr_name, type_dec.clone(), offset, class.id));
                }
                ast::FeatureNode::Method { id, .. } => {
                    match methods.iter().position(|m| m.name() == *id) {
                        Some(i) => methods[i].set_impl_class(class.id),
                        None => {
                            let slot = methods.len();
                            methods.push(MethodInfo::new(*id, class.id, slot));
                        }
                    }
                }
            }
        }

        let size = match class.id {
            INT_ID | BOOL_ID => 4,
            STRING_ID => 5,
            _ => HEADER_WORDS + attrs.len(),
        };

        self.tags.insert(class.id, tag);
        self.class_info.push(ClassInfo::from(
            class.id,
            name,
            tag,
            class.parent,
            attrs,
            methods,
            size,
        ));
    }

    pub fn class_info_of(&self, class_id: usize) -> &ClassInfo {
        self.class_info
            .iter()
            .find(|class| class.id() == class_id)
            .unwrap()
    }

    fn tag_of(&self, class_id: usize) -> usize {
        *self.tags.get(&class_id).unwrap()
    }

    pub fn codegen(&mut self) -> String {
        let mut res = String::with_capacity(64 * 1024);

        // Data section
        res.push_str(".data\n");
        res.push_str(&emit_globals());
        res.push_str(&emit_tags_builtins(
            self.tag_of(INT_ID),
            self.tag_of(BOOL_ID),
            self.tag_of(STRING_ID),
        ));
        res.push_str(&emit_gc(
            self.gc_mode.initializer(),
            self.gc_mode.collector(),
            self.gc_mode.test_flag(),
        ));

        for int in self.constants.ints() {
            res.push_str(&emit_int_const(self.tag_of(INT_ID), *int, "Int_dispTab"));
        }

        for str_id in self.constants.strings() {
            let string = self.s_table.string_from_id(*str_id).unwrap();
            res.push_str(&emit_string_const(
                self.tag_of(STRING_ID),
                string,
                *str_id,
                "String_dispTab",
            ));
        }

        res.push_str(&emit_bool_const(self.tag_of(BOOL_ID), "Bool_dispTab"));

        res
    }
}
