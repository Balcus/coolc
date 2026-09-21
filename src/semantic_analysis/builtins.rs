use crate::semantic_analysis::method_table::ReturnType;

// Builtin classes
pub const OBJECT_ID: usize = 0;
pub const INT_ID: usize = 1;
pub const BOOL_ID: usize = 2;
pub const STRING_ID: usize = 3;
pub const IO_ID: usize = 4;

// Builtin Methods
pub const ABORT_ID: usize = 5;
pub const TYPE_NAME_ID: usize = 6;
pub const COPY_ID: usize = 7;
pub const OUT_STRING_ID: usize = 8;
pub const OUT_INT_ID: usize = 9;
pub const IN_STRING_ID: usize = 10;
pub const IN_INT_ID: usize = 11;
pub const LENGTH_ID: usize = 12;
pub const CONCAT_ID: usize = 13;
pub const SUBSTR_ID: usize = 14;

// Method parameters
pub const X_ID: usize = 15;
pub const I_ID: usize = 16;
pub const L_ID: usize = 17;
pub const S_ID: usize = 18;

pub struct BuiltinMethod {
    pub name: usize,
    pub rt: ReturnType,
    // array of tuples where each tuple represents a parameter with its type
    pub params: &'static[(usize, usize)],
}

pub struct BuiltinClass {
    pub id: usize,
    pub parent: Option<usize>,
    pub methods: &'static [BuiltinMethod],
}

pub const BUILTINS: &'static [BuiltinClass] = &[
    BuiltinClass {
        id: OBJECT_ID,
        parent: None,
        methods: &[
            // Halts program execution with an error message
            BuiltinMethod {
                name: ABORT_ID,
                rt: ReturnType::Type(OBJECT_ID),
                params: &[],
            },
            // Returns a string with the name of the class of the object
            BuiltinMethod {
                name: TYPE_NAME_ID,
                rt: ReturnType::Type(STRING_ID),
                params: &[],
            },
            // Produces a shallow copy of the object
            BuiltinMethod {
                name: COPY_ID,
                rt: ReturnType::SelfType,
                params: &[],
            }
        ],
    },
    BuiltinClass {
        id: IO_ID,
        parent: Some(OBJECT_ID),
        methods: &[
            // print argument, return their self parameter
            BuiltinMethod {
                name: OUT_STRING_ID,
                rt: ReturnType::SelfType,
                params: &[(X_ID, STRING_ID)],
            },
            // print argument, return their self parameter
            BuiltinMethod {
                name: OUT_INT_ID,
                rt: ReturnType::SelfType,
                params: &[(X_ID, INT_ID)],
            },
            // reads a string from the standard input, up to but not including a newline character
            BuiltinMethod {
                name: IN_STRING_ID,
                rt: ReturnType::Type(STRING_ID),
                params: &[],
            },
            // reads a single integer, which may be preceded by whitespace
            BuiltinMethod {
                name: IN_INT_ID,
                rt: ReturnType::Type(INT_ID),
                params: &[],
            }
        ],
    },
    // Default initialization for variables of type Int is 0
    BuiltinClass {
        id: INT_ID,
        parent: Some(OBJECT_ID),
        methods: &[],
    },
    // Default initialization for variables of type String is ""
    BuiltinClass {
        id: STRING_ID,
        parent: Some(OBJECT_ID),
        methods: &[
            // returns length of self parameter
            BuiltinMethod {
                name: LENGTH_ID,
                rt: ReturnType::Type(INT_ID),
                params: &[],
            },
            // returns the string formed by concatenating s after self
            BuiltinMethod {
                name: CONCAT_ID,
                rt: ReturnType::Type(STRING_ID),
                params: &[(S_ID, STRING_ID)],
            },
            // substring of its self parameter beginning at position i with length l
            // a runtime error is generated if the specified substring is out of range
            BuiltinMethod {
                name: SUBSTR_ID,
                rt: ReturnType::Type(STRING_ID),
                params: &[(I_ID, INT_ID), (L_ID, INT_ID)],
            }
        ],
    },
    // Default initialization is false
    BuiltinClass {
        id: BOOL_ID,
        parent: Some(OBJECT_ID),
        methods: &[],
    }
];
