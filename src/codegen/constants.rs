#![allow(dead_code)]
use std::format;

pub const WORD: i32 = 4;

// --- Section: Object Layout --- //

// offset   contents
// ------   ---------------------------------
//     -4    -1: used by the GC
//      0     class tag
//      4     object size in words
//      8     pointer to class dispatch table
//     12     attribute 0 (inherited first)
//     16     attribute 1
//     ...

pub const EYECATCHER_OFFSET: i32 = -4;
pub const EYECATCHER: i32 = -1;

pub const TAG_OFFSET: i32 = 0;
pub const SIZE_OFFSET: i32 = 4;
pub const DISP_OFFSET: i32 = 8;
pub const ATTR_OFFSET: i32 = 12;
pub const HEADER_WORDS: i32 = 3;

pub const INT_SLOT: i32 = 12;
pub const BOOL_SLOT: i32 = 12;
pub const STR_SIZE: i32 = 12;
pub const STR_FIELD: i32 = 16;

pub fn attr_offset(index: usize) -> i32 {
    ATTR_OFFSET + WORD * index as i32
}

pub fn dispatch_offset(slot: usize) -> i32 {
    WORD * slot as i32
}

pub fn object_size_words(n_attrs: usize) -> usize {
    HEADER_WORDS as usize + n_attrs
}

// --- Section: Registers  --- //

pub const ACC: &str = "$a0";
pub const SELF: &str = "$s0";
pub const T1: &str = "$t1";
pub const T2: &str = "$t2";
pub const T3: &str = "$t3";
pub const A1: &str = "$a1";
pub const FP: &str = "$fp";
pub const SP: &str = "$sp";
pub const RA: &str = "$ra";
pub const ZERO: &str = "$zero";

// --- Section: Symbols defined by trap.handler  --- //

pub const OBJECT_COPY: &str = "Object.copy";
pub const OBJECT_ABORT: &str = "Object.abort";
pub const OBJECT_TYPE_NAME: &str = "Object.type_name";

pub const IO_OUT_STRING: &str = "IO.out_string";
pub const IO_OUT_INT: &str = "IO.out_int";
pub const IO_IN_STRING: &str = "IO.in_string";
pub const IO_IN_INT: &str = "IO.in_int";

pub const STRING_LENGTH: &str = "String.length";
pub const STRING_CONCAT: &str = "String.concat";
pub const STRING_SUBSTR: &str = "String.substr";

pub const EQUALITY_TEST: &str = "equality_test";
pub const DISPATCH_ABORT: &str = "_dispatch_abort";
pub const CASE_ABORT: &str = "_case_abort";
pub const CASE_ABORT2: &str = "_case_abort2";

pub const NOGC_INIT: &str = "_NoGC_Init";
pub const NOGC_COLLECT: &str = "_NoGC_Collect";
pub const GENGC_INIT: &str = "_GenGC_Init";
pub const GENGC_COLLECT: &str = "_GenGC_Collect";
pub const GENGC_ASSIGN: &str = "_GenGC_Assign";

// --- Section: Symbols trap.handler expects (must be defined in codegen)  --- //

pub const CLASS_NAME_TAB: &str = "class_nameTab";
pub const CLASS_OBJ_TAB: &str = "class_objTab";
pub const INT_TAG: &str = "_int_tag";
pub const BOOL_TAG: &str = "_bool_tag";
pub const STRING_TAG: &str = "_string_tag";
pub const BOOL_CONST0: &str = "bool_const0";
pub const BOOL_CONST1: &str = "bool_const1";
pub const HEAP_START: &str = "heap_start";
pub const MEMMGR_INITIALIZER: &str = "_MemMgr_INITIALIZER";
pub const MEMMGR_COLLECTOR: &str = "_MemMgr_COLLECTOR";
pub const MEMMGR_TEST: &str = "_MemMgr_TEST";

// Symbols that must be declared in .globl

pub const REQUIRED_GLOBALS: &[&str] = &[
    CLASS_NAME_TAB,
    "Main_protObj",
    "Int_protObj",
    "String_protObj",
    BOOL_CONST0,
    BOOL_CONST1,
    INT_TAG,
    BOOL_TAG,
    STRING_TAG,
    HEAP_START,
    MEMMGR_INITIALIZER,
    MEMMGR_COLLECTOR,
    MEMMGR_TEST,
    "Main_init",
    "Int_init",
    "Bool_init",
    "String_init",
    "Main.main",
];

// --- Section: GC  --- //

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcMode {
    NoGc,
    GenGc,
    GenGcTest,
}

impl GcMode {
    pub fn initializer(self) -> &'static str {
        match self {
            GcMode::NoGc => NOGC_INIT,
            _ => GENGC_INIT,
        }
    }

    pub fn collector(self) -> &'static str {
        match self {
            GcMode::NoGc => NOGC_COLLECT,
            _ => GENGC_COLLECT,
        }
    }

    pub fn test_flag(self) -> i32 {
        matches!(self, GcMode::GenGcTest) as i32
    }
}

// --- Section: Built-in class method lists in dispatch table order --- //

pub const OBJECT_METHODS: &[&str] = &["abort", "type_name", "copy"];
pub const IO_METHODS: &[&str] = &["out_string", "out_int", "in_string", "in_int"];
pub const STRING_METHODS: &[&str] = &["length", "concat", "substr"];

// --- Section: Label builders --- //

pub fn method_label(class: &str, method: &str) -> String {
    format!("{class}.{method}")
}

pub fn init_label(class: &str) -> String {
    format!("{class}_init")
}

pub fn protobj_label(class: &str) -> String {
    format!("{class}_protObj")
}

pub fn disptab_label(class: &str) -> String {
    format!("{class}_dispTab")
}

pub fn int_const_label(n: usize) -> String {
    format!("int_const{n}")
}

pub fn str_const_label(n: usize) -> String {
    format!("str_const{n}")
}
