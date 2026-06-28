use coolc::{
    ast,
    lexer::{ErrorToken, LexerWrapper, Token},
    grammar,
    s_table::StringTable,
};
use lalrpop_util::{ErrorRecovery, ParseError};

fn parse(
    input: &str,
    s_table: &mut StringTable,
    errors: &mut Vec<ErrorRecovery<usize, Token, ErrorToken>>,
) -> Result<ast::Program, ParseError<usize, Token, ErrorToken>> {
    let program = grammar::ProgramParser::new().parse(errors, LexerWrapper::new(input, s_table, String::from("test")))?;

    if !errors.is_empty() {
        return Err(errors[0].error.clone());
    }

    Ok(program)
}

fn i(s_table: &mut StringTable, s: &str) -> usize {
    s_table.insert(s.to_string())
}

mod succeds_parsing {
    use super::*;
    use test_case::test_case;

    #[test]
    fn one_class_one_constant_attribute() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            x: Int <- 10;
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Attribute {
                    name: i(&mut s_table, "x"),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    init: Some(Box::new(ast::Expr::IntConstant(10))),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn one_class_many_constant_attributes() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            x: Int <- 10;
            y: String <- "Hello World";
            z: Bool <- false;
            a: Int;
            b: String;
            c: Bool;
            d: IO;
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let hello_world = i(&mut s_table, "Hello World");
        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "x"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                        init: Some(Box::new(ast::Expr::IntConstant(10))),
                    },
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "y"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "String")),
                        init: Some(Box::new(ast::Expr::StringConstant(hello_world))),
                    },
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "z"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                        init: Some(Box::new(ast::Expr::BoolConstant(false))),
                    },
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "a"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                        init: None,
                    },
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "b"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "String")),
                        init: None,
                    },
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "c"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                        init: None,
                    },
                    ast::Feature::Attribute {
                        name: i(&mut s_table, "d"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "IO")),
                        init: None,
                    },
                ],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn class_inheritance() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {};
        class A {};
        class B inherits A {};
        class C inherits B {};
        class D inherits A {};
        "#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![
                ast::Class::Valid {
                    name: i(&mut s_table, "Main"),
                    parent: None,
                    features: Vec::new(),
                },
                ast::Class::Valid {
                    name: i(&mut s_table, "A"),
                    parent: None,
                    features: Vec::new(),
                },
                ast::Class::Valid {
                    name: i(&mut s_table, "B"),
                    parent: Some(i(&mut s_table, "A")),
                    features: Vec::new(),
                },
                ast::Class::Valid {
                    name: i(&mut s_table, "C"),
                    parent: Some(i(&mut s_table, "B")),
                    features: Vec::new(),
                },
                ast::Class::Valid {
                    name: i(&mut s_table, "D"),
                    parent: Some(i(&mut s_table, "A")),
                    features: Vec::new(),
                },
            ],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn multiple_classes_constant_attributes() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            x: Int <- 1;
        };
        class Test {
            y: Bool <- true;
        };
        class Test2 {
            z: String <- "String";
        };
        "#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let string_id = i(&mut s_table, "String");
        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![
                ast::Class::Valid {
                    name: i(&mut s_table, "Main"),
                    parent: None,
                    features: vec![ast::Feature::Attribute {
                        name: i(&mut s_table, "x"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                        init: Some(Box::new(ast::Expr::IntConstant(1))),
                    }],
                },
                ast::Class::Valid {
                    name: i(&mut s_table, "Test"),
                    parent: None,
                    features: vec![ast::Feature::Attribute {
                        name: i(&mut s_table, "y"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                        init: Some(Box::new(ast::Expr::BoolConstant(true))),
                    }],
                },
                ast::Class::Valid {
                    name: i(&mut s_table, "Test2"),
                    parent: None,
                    features: vec![ast::Feature::Attribute {
                        name: i(&mut s_table, "z"),
                        type_dec: ast::TypeName::Type(string_id),
                        init: Some(Box::new(ast::Expr::StringConstant(string_id))),
                    }],
                },
            ],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn method_no_params() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            doStuff(): Int { 42 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "doStuff"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::IntConstant(42)),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn method_with_params() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            isNull(o: Object): Bool { false };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "isNull"),
                    params: vec![ast::Formal {
                        name: i(&mut s_table, "o"),
                        type_dec: i(&mut s_table, "Object"),
                    }],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::BoolConstant(false)),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn method_with_body_and_params() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            add(x: Int): Int { 42 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "add"),
                    params: vec![ast::Formal {
                        name: i(&mut s_table, "x"),
                        type_dec: i(&mut s_table, "Int"),
                    }],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::IntConstant(42)),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn method_with_assignment() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            changeValue(from: Int, to: Int): Int { from <- to };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let to_id = i(&mut s_table, "to");
        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "changeValue"),
                    params: vec![
                        ast::Formal {
                            name: i(&mut s_table, "from"),
                            type_dec: i(&mut s_table, "Int"),
                        },
                        ast::Formal {
                            name: to_id,
                            type_dec: i(&mut s_table, "Int"),
                        },
                    ],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Assignment {
                        var: ast::Var::Id(i(&mut s_table, "from")),
                        expr: Box::new(ast::Expr::Object(to_id)),
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn conditional_basic() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { if true then 1 else 0 fi };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Conditional {
                        cond: Box::new(ast::Expr::BoolConstant(true)),
                        happy_path: Box::new(ast::Expr::IntConstant(1)),
                        sad_path: Box::new(ast::Expr::IntConstant(0)),
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn while_loop_basic() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { while true loop 1 pool };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Loop {
                        cond: Box::new(ast::Expr::BoolConstant(true)),
                        body: Box::new(ast::Expr::IntConstant(1)),
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn block_single_expr() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { { 42; } };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Block(vec![ast::Expr::IntConstant(42)])),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn block_multiple_exprs() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { { 1; 2; 3; } };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Block(vec![
                        ast::Expr::IntConstant(1),
                        ast::Expr::IntConstant(2),
                        ast::Expr::IntConstant(3),
                    ])),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn new_object() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Object { new Object };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Object")),
                    body: Box::new(ast::Expr::New(ast::TypeName::Type(i(
                        &mut s_table,
                        "Object",
                    )))),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn isvoid_expr() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool { isvoid 42 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::IsVoid(Box::new(ast::Expr::IntConstant(42)))),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn negation_expr() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { ~42 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Neg(Box::new(ast::Expr::IntConstant(42)))),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn not_expr() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool { not true };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::Not(Box::new(ast::Expr::BoolConstant(true)))),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn arithmetic_add() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { 1 + 2 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Add(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::IntConstant(2)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn arithmetic_sub() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { 5 - 3 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Sub(
                        Box::new(ast::Expr::IntConstant(5)),
                        Box::new(ast::Expr::IntConstant(3)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn arithmetic_mul() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { 3 * 4 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Mul(
                        Box::new(ast::Expr::IntConstant(3)),
                        Box::new(ast::Expr::IntConstant(4)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn arithmetic_div() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { 10 / 2 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Div(
                        Box::new(ast::Expr::IntConstant(10)),
                        Box::new(ast::Expr::IntConstant(2)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn comparison_lt() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool { 1 < 2 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::Lt(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::IntConstant(2)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn comparison_le() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool { 1 <= 2 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::Le(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::IntConstant(2)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn comparison_gt() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool {
                2 > 1
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: Vec::new(),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::Lt(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::IntConstant(2)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn comparison_ge() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool {
                2 >= 1
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: Vec::new(),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::Le(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::IntConstant(2)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn comparison_eq() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Bool { 1 = 1 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                    body: Box::new(ast::Expr::Eq(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::IntConstant(1)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn case_single_branch() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { case 42 of x: Int => 1; esac };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Case {
                        cond: Box::new(ast::Expr::IntConstant(42)),
                        branches: vec![ast::CaseBranch {
                            name: i(&mut s_table, "x"),
                            type_dec: i(&mut s_table, "Int"),
                            body: Box::new(ast::Expr::IntConstant(1)),
                        }],
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn case_multiple_branches() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { case 42 of x: Int => 1; y: Bool => 2; esac };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Case {
                        cond: Box::new(ast::Expr::IntConstant(42)),
                        branches: vec![
                            ast::CaseBranch {
                                name: i(&mut s_table, "x"),
                                type_dec: i(&mut s_table, "Int"),
                                body: Box::new(ast::Expr::IntConstant(1)),
                            },
                            ast::CaseBranch {
                                name: i(&mut s_table, "y"),
                                type_dec: i(&mut s_table, "Bool"),
                                body: Box::new(ast::Expr::IntConstant(2)),
                            },
                        ],
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn precedence_mul_over_add() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { 1 + 2 * 3 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Add(
                        Box::new(ast::Expr::IntConstant(1)),
                        Box::new(ast::Expr::Mul(
                            Box::new(ast::Expr::IntConstant(2)),
                            Box::new(ast::Expr::IntConstant(3)),
                        )),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn precedence_parens_override() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            test(): Int { (1 + 2) * 3 };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "test"),
                    params: vec![],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Mul(
                        Box::new(ast::Expr::Add(
                            Box::new(ast::Expr::IntConstant(1)),
                            Box::new(ast::Expr::IntConstant(2)),
                        )),
                        Box::new(ast::Expr::IntConstant(3)),
                    )),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn examples_hello_world_cl() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = include_str!("../examples/hello_world.cl");

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: Some(i(&mut s_table, "IO")),
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "main"),
                    params: Vec::new(),
                    type_dec: ast::TypeName::SelfType,
                    body: Box::new(ast::Expr::SelfDispatch {
                        name: i(&mut s_table, "out_string"),
                        args: vec![ast::Expr::StringConstant(i(
                            &mut s_table,
                            "Hello, World.\n",
                        ))],
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn simple_let() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main(): Int {
                plus(1, 2)
            };

            plus(num1: Int, num2: Int): Int {
                let x: Int in
                {
                    x <- num1 + num2;
                    x;
                }
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![
                    ast::Feature::Method {
                        name: i(&mut s_table, "main"),
                        params: Vec::new(),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                        body: Box::new(ast::Expr::SelfDispatch {
                            name: i(&mut s_table, "plus"),
                            args: vec![ast::Expr::IntConstant(1), ast::Expr::IntConstant(2)],
                        }),
                    },
                    ast::Feature::Method {
                        name: i(&mut s_table, "plus"),
                        params: vec![
                            ast::Formal {
                                name: i(&mut s_table, "num1"),
                                type_dec: i(&mut s_table, "Int"),
                            },
                            ast::Formal {
                                name: i(&mut s_table, "num2"),
                                type_dec: i(&mut s_table, "Int"),
                            },
                        ],
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                        body: Box::new(ast::Expr::Let {
                            name: i(&mut s_table, "x"),
                            type_dec: i(&mut s_table, "Int"),
                            init: None,
                            body: Box::new(ast::Expr::Block(vec![
                                ast::Expr::Assignment {
                                    var: ast::Var::Id(i(&mut s_table, "x")),
                                    expr: Box::new(ast::Expr::Add(
                                        Box::new(ast::Expr::Object(i(&mut s_table, "num1"))),
                                        Box::new(ast::Expr::Object(i(&mut s_table, "num2"))),
                                    )),
                                },
                                ast::Expr::Object(i(&mut s_table, "x")),
                            ])),
                        }),
                    },
                ],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn multi_binding_let() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main(): Int {
                let x: Int, y: Int <- 5, z: Bool in
                {
                    x <- 1;
                    y;
                }
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "main"),
                    params: Vec::new(),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Let {
                        name: i(&mut s_table, "x"),
                        type_dec: i(&mut s_table, "Int"),
                        init: None,
                        body: Box::new(ast::Expr::Let {
                            name: i(&mut s_table, "y"),
                            type_dec: i(&mut s_table, "Int"),
                            init: Some(Box::new(ast::Expr::IntConstant(5))),
                            body: Box::new(ast::Expr::Let {
                                name: i(&mut s_table, "z"),
                                type_dec: i(&mut s_table, "Bool"),
                                init: None,
                                body: Box::new(ast::Expr::Block(vec![
                                    ast::Expr::Assignment {
                                        var: ast::Var::Id(i(&mut s_table, "x")),
                                        expr: Box::new(ast::Expr::IntConstant(1)),
                                    },
                                    ast::Expr::Object(i(&mut s_table, "y")),
                                ])),
                            }),
                        }),
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn let_extends_rightmost() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main(): Int {
                let x: Int <- 1 in
                let y: Int <- 2 in
                    x + y
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "main"),
                    params: Vec::new(),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Let {
                        name: i(&mut s_table, "x"),
                        type_dec: i(&mut s_table, "Int"),
                        init: Some(Box::new(ast::Expr::IntConstant(1))),
                        body: Box::new(ast::Expr::Let {
                            name: i(&mut s_table, "y"),
                            type_dec: i(&mut s_table, "Int"),
                            init: Some(Box::new(ast::Expr::IntConstant(2))),
                            body: Box::new(ast::Expr::Add(
                                Box::new(ast::Expr::Object(i(&mut s_table, "x"))),
                                Box::new(ast::Expr::Object(i(&mut s_table, "y"))),
                            )),
                        }),
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn let_body_extends_through_block_statement() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main(): Int {
                {
                    let x: Int <- 1 in
                    let y: Int <- 2 in
                        x + y;
                    3;
                }
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "main"),
                    params: Vec::new(),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    body: Box::new(ast::Expr::Block(vec![
                        ast::Expr::Let {
                            name: i(&mut s_table, "x"),
                            type_dec: i(&mut s_table, "Int"),
                            init: Some(Box::new(ast::Expr::IntConstant(1))),
                            body: Box::new(ast::Expr::Let {
                                name: i(&mut s_table, "y"),
                                type_dec: i(&mut s_table, "Int"),
                                init: Some(Box::new(ast::Expr::IntConstant(2))),
                                body: Box::new(ast::Expr::Add(
                                    Box::new(ast::Expr::Object(i(&mut s_table, "x"))),
                                    Box::new(ast::Expr::Object(i(&mut s_table, "y"))),
                                )),
                            }),
                        },
                        ast::Expr::IntConstant(3),
                    ])),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test]
    fn let_as_first_of_multiple_block_statements() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main(): Int {
                {
                    let x: Int <- 1 in x;
                    3;
                }
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());
    }

    #[test]
    fn paren_let() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            method3(num : Int) : C {
                (let x : Int in
                    {
                        x <- ~num;
                        (new C).set_var(x);
                    }
                )
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());
    }

    #[test]
    fn op_as_dispatch_param() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main(s: String) : String {
                s.substr(1, s.length() - 2);
            };
        };"#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());
    }

    #[test]
    fn let_inside_assignment() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            main() : Int {
                y <- let x : Int <- 5 in x + 1;
            };
        };
        "#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());
    }

    #[test]
    fn adding_if_results() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
        class Main {
            cell_at_next_evolution(position : Int) : String {
                if (if cell(position) = "X" then 1 else 0 fi
                    + if cell_left_neighbor(position) = "X" then 1 else 0 fi
                    + if cell_right_neighbor(position) = "X" then 1 else 0 fi
                    = 1)
                then
                    "X"
                else
                    "."
                fi
            };
        };
        "#;

        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());

        let mut errors = Vec::new();
        let expected = ast::Program {
            classes: vec![ast::Class::Valid {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Method {
                    name: i(&mut s_table, "cell_at_next_evolution"),
                    params: vec![ast::Formal {
                        name: i(&mut s_table, "position"),
                        type_dec: i(&mut s_table, "Int"),
                    }],
                    type_dec: ast::TypeName::Type(i(&mut s_table, "String")),
                    body: Box::new(ast::Expr::Conditional {
                        cond: Box::new(ast::Expr::Eq(
                            Box::new(ast::Expr::Add(
                                Box::new(ast::Expr::Add(
                                    Box::new(ast::Expr::Conditional {
                                        cond: Box::new(ast::Expr::Eq(
                                            Box::new(ast::Expr::SelfDispatch {
                                                name: i(&mut s_table, "cell"),
                                                args: vec![ast::Expr::Object(i(
                                                    &mut s_table,
                                                    "position",
                                                ))],
                                            }),
                                            Box::new(ast::Expr::StringConstant(i(
                                                &mut s_table,
                                                "X",
                                            ))),
                                        )),
                                        happy_path: Box::new(ast::Expr::IntConstant(1)),
                                        sad_path: Box::new(ast::Expr::IntConstant(0)),
                                    }),
                                    Box::new(ast::Expr::Conditional {
                                        cond: Box::new(ast::Expr::Eq(
                                            Box::new(ast::Expr::SelfDispatch {
                                                name: i(&mut s_table, "cell_left_neighbor"),
                                                args: vec![ast::Expr::Object(i(
                                                    &mut s_table,
                                                    "position",
                                                ))],
                                            }),
                                            Box::new(ast::Expr::StringConstant(i(
                                                &mut s_table,
                                                "X",
                                            ))),
                                        )),
                                        happy_path: Box::new(ast::Expr::IntConstant(1)),
                                        sad_path: Box::new(ast::Expr::IntConstant(0)),
                                    }),
                                )),
                                Box::new(ast::Expr::Conditional {
                                    cond: Box::new(ast::Expr::Eq(
                                        Box::new(ast::Expr::SelfDispatch {
                                            name: i(&mut s_table, "cell_right_neighbor"),
                                            args: vec![ast::Expr::Object(i(
                                                &mut s_table,
                                                "position",
                                            ))],
                                        }),
                                        Box::new(ast::Expr::StringConstant(i(&mut s_table, "X"))),
                                    )),
                                    happy_path: Box::new(ast::Expr::IntConstant(1)),
                                    sad_path: Box::new(ast::Expr::IntConstant(0)),
                                }),
                            )),
                            Box::new(ast::Expr::IntConstant(1)),
                        )),
                        happy_path: Box::new(ast::Expr::StringConstant(i(&mut s_table, "X"))),
                        sad_path: Box::new(ast::Expr::StringConstant(i(&mut s_table, "."))),
                    }),
                }],
            }],
        };

        assert_eq!(parse(input, &mut s_table, &mut errors).unwrap(), expected);
        assert!(errors.is_empty());
    }

    #[test_case("arith.cl", include_str!("../examples/arith.cl"); "arith")]
    #[test_case("atoi.cl", include_str!("../examples/atoi.cl"); "atoi")]
    #[test_case("atoi_test.cl", include_str!("../examples/atoi_test.cl"); "atoi_test")]
    #[test_case("book_list.cl", include_str!("../examples/book_list.cl"); "book_list")]
    #[test_case("cells.cl", include_str!("../examples/cells.cl"); "cells")]
    #[test_case("complex.cl", include_str!("../examples/complex.cl"); "complex")]
    #[test_case("cool.cl", include_str!("../examples/cool.cl"); "cool")]
    #[test_case("hairyscary.cl", include_str!("../examples/hairyscary.cl"); "hairyscary")]
    #[test_case("hello_world.cl", include_str!("../examples/hello_world.cl"); "hello_world")]
    #[test_case("io.cl", include_str!("../examples/io.cl"); "io")]
    #[test_case("lam.cl", include_str!("../examples/lam.cl"); "lam")]
    #[test_case("life.cl", include_str!("../examples/life.cl"); "life")]
    #[test_case("list.cl", include_str!("../examples/list.cl"); "list")]
    #[test_case("new_complex.cl", include_str!("../examples/new_complex.cl"); "new_complex")]
    #[test_case("palindrome.cl", include_str!("../examples/palindrome.cl"); "palindrome")]
    #[test_case("primes.cl", include_str!("../examples/primes.cl"); "primes")]
    #[test_case("sort_list.cl", include_str!("../examples/sort_list.cl"); "sort_list")]
    fn parses_examples(_: &str, input: &str) {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        assert!(parse(input, &mut s_table, &mut errors).is_ok());
        assert!(errors.is_empty());
    }
}

mod fail_parsing {
    use coolc::lexer::{ErrorKind, Span};

    use super::*;

    #[test]
    fn empty_string() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#""#;

        assert!(parse(input, &mut s_table, &mut errors).is_err());
    }

    #[test]
    fn error_recovery_invalid_class() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();

        let input = r#"
            class main {

            };

            class foo {

            };

            class bar {

            };

            class Main {
                x: Int <- 10;
            };

            class foobar {

            };
        "#;

        assert!(parse(input, &mut s_table, &mut errors).is_err());
        assert_eq!(errors.len(), 4);
        assert!(matches!(
            errors[0].error,
            lalrpop_util::ParseError::UnrecognizedToken { .. }
        ));
        assert!(matches!(
            errors[1].error,
            lalrpop_util::ParseError::UnrecognizedToken { .. }
        ));
        assert!(matches!(
            errors[2].error,
            lalrpop_util::ParseError::UnrecognizedToken { .. }
        ));
        assert!(matches!(
            errors[3].error,
            lalrpop_util::ParseError::UnrecognizedToken { .. }
        ));
    }

    #[test]
    fn lexing_error_unterminated_string() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = "class Main {
            x: String <- \"This is \n a string\";
        };";

        let result = parse(input, &mut s_table, &mut errors);

        match result {
            Err(lalrpop_util::ParseError::User { error }) => {
                assert_eq!(
                    error,
                    ErrorToken::new(
                        ErrorKind::UnterminatedStringConstant,
                        String::from("Unterminated string constant"),
                        Span::new("test".to_string(),  38, 47)
                    )
                );
            }
            _ => panic!("Expected lexer error"),
        }
    }
}
