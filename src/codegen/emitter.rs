use std::{fmt::Write, format};

use crate::codegen::{
    TagGen,
    constants::{EYECATCHER, REQUIRED_GLOBALS},
};

pub fn emit_globals() -> String {
    let mut to = String::new();
    for global in REQUIRED_GLOBALS {
        writeln!(to, "        .globl  {}", global).unwrap();
    }
    to
}

pub fn emit_proto_builtins(tag_gen: &mut TagGen) -> String {
    let object = tag_gen.get();
    let io = tag_gen.get();
    let int = tag_gen.get();
    let bool_ = tag_gen.get();
    let string = tag_gen.get();

    format!(
        concat!(
            "        .word   {eyecatcher}\n",
            "Object_protObj:\n",
            "        .word   {object}\n",
            "        .word   3\n",
            "        .word   Object_dispTab\n",
            "        .word   {eyecatcher}\n",
            "IO_protObj:\n",
            "        .word   {io}\n",
            "        .word   3\n",
            "        .word   IO_dispTab\n",
            "        .word   {eyecatcher}\n",
            "Int_protObj:\n",
            "        .word   {int}\n",
            "        .word   4\n",
            "        .word   Int_dispTab\n",
            "        .word   0\n",
            "        .word   {eyecatcher}\n",
            "Bool_protObj:\n",
            "        .word   {bool_}\n",
            "        .word   4\n",
            "        .word   Bool_dispTab\n",
            "        .word   0\n",
            "        .word   {eyecatcher}\n",
            "String_protObj:\n",
            "        .word   {string}\n",
            "        .word   5\n",
            "        .word   String_dispTab\n",
            "        .word   int_const0\n",
            "        .word   0\n",
        ),
        eyecatcher = EYECATCHER,
        object = object,
        io = io,
        int = int,
        bool_ = bool_,
        string = string,
    )
}

#[cfg(test)]
mod test {
    use super::*;

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
            "        .globl  Main.main\n",
        );
        assert_eq!(emit_globals(), expected);
    }
}
