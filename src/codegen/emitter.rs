use std::{fmt::Write, format, writeln};

use crate::{
    builtins::{BOOL_ID, INT_ID, STRING_ID},
    codegen::{
        class_info::ClassInfo,
        constants::{
            BOOL_TAG_LABEL, EYECATCHER, INT_TAG_LABEL, MEMMGR_COLLECTOR, MEMMGR_INITIALIZER,
            MEMMGR_TEST, REQUIRED_GLOBALS, STRING_TAG_LABEL,
        },
    },
    utils::Type,
};

pub fn emit_globals() -> String {
    let mut res = String::new();
    for global in REQUIRED_GLOBALS {
        writeln!(res, "        .globl  {}", global).unwrap();
    }
    writeln!(res, "").unwrap();
    res
}

pub fn emit_int_const(int_tag: usize, value: i64, disp: &str) -> String {
    format!(
        concat!(
            "        .word   {eyecatcher}\n",
            "int_const{label}:\n",
            "        .word   {tag}\n",
            "        .word   {size}\n",
            "        .word   {disp}\n",
            "        .word   {value}\n\n",
        ),
        eyecatcher = EYECATCHER,
        label = value,
        tag = int_tag,
        size = 4,
        disp = disp,
        value = value
    )
}

pub fn emit_string_const(string_tag: usize, value: &str, id: usize, disp: &str) -> String {
    format!(
        concat!(
            "        .word   {eyecatcher}\n",
            "str_const{id}:\n",
            "        .word   {tag}\n",
            "        .word   {size}\n",
            "        .word   {disp}\n",
            "        .word   {len}\n",
            "        .asciiz \"{value}\"\n",
            "        .align 2\n\n",
        ),
        eyecatcher = EYECATCHER,
        id = id,
        tag = string_tag,
        size = 4 + (value.len() + 4) / 4,
        disp = disp,
        len = format!("int_const{}", &value.len()),
        value = value
    )
}

pub fn emit_bool_const(bool_tag: usize, disp: &str) -> String {
    format!(
        concat!(
            "        .word   {eyecatcher}\n",
            "bool_const0:\n",
            "        .word   {tag}\n",
            "        .word   4\n",
            "        .word   {disp}\n",
            "        .word   0\n\n",
            "        .word   {eyecatcher}\n",
            "bool_const1:\n",
            "        .word   {tag}\n",
            "        .word   4\n",
            "        .word   {disp}\n",
            "        .word   1\n\n",
        ),
        eyecatcher = EYECATCHER,
        tag = bool_tag,
        disp = disp
    )
}

pub fn emit_tags_builtins(int_tag: usize, bool_tag: usize, string_tag: usize) -> String {
    format!(
        concat!(
            "{int_label}:\n",
            "        .word   {int}\n",
            "{bool_label}:\n",
            "        .word   {bool_}\n",
            "{string_label}:\n",
            "        .word   {string}\n\n",
        ),
        int_label = INT_TAG_LABEL,
        bool_label = BOOL_TAG_LABEL,
        string_label = STRING_TAG_LABEL,
        int = int_tag,
        bool_ = bool_tag,
        string = string_tag,
    )
}

pub fn emit_gc(gc_init: &str, gc_collector: &str, gc_test: i32) -> String {
    format!(
        concat!(
            "{mem_init}:\n",
            "        .word  {init}\n",
            "{mem_collector}:\n",
            "        .word  {collector}\n",
            "{mem_test}:\n",
            "        .word {test_flag}\n\n",
        ),
        mem_init = MEMMGR_INITIALIZER,
        mem_collector = MEMMGR_COLLECTOR,
        mem_test = MEMMGR_TEST,
        init = gc_init,
        collector = gc_collector,
        test_flag = gc_test,
    )
}

pub fn emit_proto_obj(class_info: &ClassInfo) -> String {
    let mut proto = format!(
        concat!(
            "        .word   {eyecatcher}\n",
            "{label}:\n",
            "        .word   {tag}\n",
            "        .word   {size}\n",
            "        .word   {disp}\n",
        ),
        eyecatcher = EYECATCHER,
        label = class_info.proto_obj_label(),
        tag = class_info.tag(),
        size = class_info.size(),
        disp = class_info.disp_label(),
    );

    for attr in class_info.attrs() {
        let default = match attr.ty() {
            Type::Fixed(id) if *id == INT_ID => "int_const0",
            Type::Fixed(id) if *id == BOOL_ID => "bool_const0",
            Type::Fixed(id) if *id == STRING_ID => "str_const0",
            Type::Fixed(_) | Type::SelfType => "0",
        };
        proto.push_str(&format!("        .word   {}\n", default));
    }

    proto
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builtins::OBJECT_ID;
    use crate::codegen::AttrInfo;

    #[test]
    fn check_globals_emitter() {
        let expected = concat!(
            "        .globl  class_nameTab\n",
            "        .globl  Main_protObj\n",
            "        .globl  Int_protObj\n",
            "        .globl  String_protObj\n",
            "        .globl  bool_const0\n",
            "        .globl  bool_const1\n",
            "        .globl  _int_tag\n",
            "        .globl  _bool_tag\n",
            "        .globl  _string_tag\n",
            "        .globl  heap_start\n",
            "        .globl  _MemMgr_INITIALIZER\n",
            "        .globl  _MemMgr_COLLECTOR\n",
            "        .globl  _MemMgr_TEST\n",
            "        .globl  Main_init\n",
            "        .globl  Int_init\n",
            "        .globl  Bool_init\n",
            "        .globl  String_init\n",
            "        .globl  Main.main\n\n",
        );
        assert_eq!(emit_globals(), expected);
    }

    // class Foo {
    //     first : Object;
    //     second : Object;
    // };
    #[test]
    fn check_codegen_class() {
        let foo = ClassInfo::from(
            5,
            String::from("Foo"),
            5,
            Some(OBJECT_ID),
            vec![
                AttrInfo::new(String::from("first"), Type::Fixed(OBJECT_ID), 12, 5),
                AttrInfo::new(String::from("second"), Type::Fixed(OBJECT_ID), 16, 5),
            ],
            Vec::new(),
            5,
        );

        let expected = concat!(
            "        .word   -1\n",
            "Foo_protObj:\n",
            "        .word   5\n",
            "        .word   5\n",
            "        .word   Foo_dispTab\n",
            "        .word   0\n",
            "        .word   0\n",
        );

        assert_eq!(emit_proto_obj(&foo), expected);
    }
}
