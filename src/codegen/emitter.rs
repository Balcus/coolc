use std::fmt::Write;

use crate::codegen::constants::REQUIRED_GLOBALS;

pub fn emit_globals() -> String {
    let mut to = String::new();
    for global in REQUIRED_GLOBALS {
        writeln!(to, "        .globl  {}", global).unwrap();
    }
    to
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
