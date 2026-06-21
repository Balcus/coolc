use coolc::{ast, lexer::LexerWrapper, parser, s_table::StringTable};

fn parse(input: &str, s_table: &mut StringTable) -> Result<ast::Program, impl std::fmt::Debug> {
    parser::ProgramParser::new().parse(LexerWrapper::new(input, s_table))
}

fn i(s_table: &mut StringTable, s: &str) -> usize {
    s_table.insert(s.to_string())
}

mod succeds_parsing {
    use super::*;

    #[test]
    fn one_class_one_constant_attribute() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            x: Int <- 10;
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
                name: i(&mut s_table, "Main"),
                parent: None,
                features: vec![ast::Feature::Attribute {
                    name: i(&mut s_table, "x"),
                    type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                    init: Some(Box::new(ast::Expr::IntConstant(10))),
                }],
            }],
        };

        assert_eq!(res, expected);
    }

    #[test]
    fn one_class_many_constant_attributes() {
        let mut s_table = StringTable::new();
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

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let hello_world = i(&mut s_table, "Hello World");
        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn class_inheritance() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {};
        class A {};
        class B inherits A {};
        class C inherits B {};
        class D inherits A {};
        "#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![
                ast::Class {
                    name: i(&mut s_table, "Main"),
                    parent: None,
                    features: Vec::new(),
                },
                ast::Class {
                    name: i(&mut s_table, "A"),
                    parent: None,
                    features: Vec::new(),
                },
                ast::Class {
                    name: i(&mut s_table, "B"),
                    parent: Some(i(&mut s_table, "A")),
                    features: Vec::new(),
                },
                ast::Class {
                    name: i(&mut s_table, "C"),
                    parent: Some(i(&mut s_table, "B")),
                    features: Vec::new(),
                },
                ast::Class {
                    name: i(&mut s_table, "D"),
                    parent: Some(i(&mut s_table, "A")),
                    features: Vec::new(),
                },
            ],
        };

        assert_eq!(res, expected);
    }

    #[test]
    fn multiple_classes_constant_attributes() {
        let mut s_table = StringTable::new();
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

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let string_id = i(&mut s_table, "String");
        let expected = ast::Program {
            classes: vec![
                ast::Class {
                    name: i(&mut s_table, "Main"),
                    parent: None,
                    features: vec![ast::Feature::Attribute {
                        name: i(&mut s_table, "x"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Int")),
                        init: Some(Box::new(ast::Expr::IntConstant(1))),
                    }],
                },
                ast::Class {
                    name: i(&mut s_table, "Test"),
                    parent: None,
                    features: vec![ast::Feature::Attribute {
                        name: i(&mut s_table, "y"),
                        type_dec: ast::TypeName::Type(i(&mut s_table, "Bool")),
                        init: Some(Box::new(ast::Expr::BoolConstant(true))),
                    }],
                },
                ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn method_no_params() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            doStuff(): Int { 42 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn method_with_params() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            isNull(o: Object): Bool { false };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn method_with_body_and_params() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            add(x: Int): Int { 42 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn method_with_assignment() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            changeValue(from: Int, to: Int): Int { from <- to };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let to_id = i(&mut s_table, "to");
        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn conditional_basic() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { if true then 1 else 0 fi };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn while_loop_basic() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { while true loop 1 pool };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn block_single_expr() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { { 42; } };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn block_multiple_exprs() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { { 1; 2; 3; } };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn new_object() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Object { new Object };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn isvoid_expr() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool { isvoid 42 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn negation_expr() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { ~42 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn not_expr() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool { not true };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn arithmetic_add() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { 1 + 2 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn arithmetic_sub() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { 5 - 3 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn arithmetic_mul() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { 3 * 4 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn arithmetic_div() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { 10 / 2 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn comparison_lt() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool { 1 < 2 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn comparison_le() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool { 1 <= 2 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn comparison_gt() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool {
                2 > 1
            };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn comparison_ge() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool {
                2 >= 1
            };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn comparison_eq() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Bool { 1 = 1 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn case_single_branch() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { case 42 of x: Int => 1; esac };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn case_multiple_branches() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { case 42 of x: Int => 1; y: Bool => 2; esac };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn precedence_mul_over_add() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { 1 + 2 * 3 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn precedence_parens_override() {
        let mut s_table = StringTable::new();
        let input = r#"
        class Main {
            test(): Int { (1 + 2) * 3 };
        };"#;

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }

    #[test]
    fn examples_hello_world_cl() {
        let mut s_table = StringTable::new();
        let input = include_str!("../examples/hello_world.cl");

        assert!(parse(input, &mut s_table).is_ok());
        let res = parse(input, &mut s_table).unwrap();

        let expected = ast::Program {
            classes: vec![ast::Class {
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

        assert_eq!(res, expected);
    }
}

mod fail_parsing {
    use super::*;

    #[test]
    fn empty_string() {
        let mut s_table = StringTable::new();
        let input = r#""#;

        assert!(parse(input, &mut s_table).is_err());
    }
}
