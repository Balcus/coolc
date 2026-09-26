mod succeeds_type_check {
    use coolc::semantic_analysis::builtins::{BOOL_ID, INT_ID, STRING_ID};
    use coolc::utils::ReturnType;
    use coolc::{ast, semantic_analysis::SemanticAnalyzer, utils::parse_program};
    use core::panic;
    use std::{assert_eq, print, vec};
    use test_case::test_case;

    #[test]
    fn object_inheritance() {
        let (_, program) = parse_program(
            r#"
            class Main inherits Object {
                x: Int <- 1;
                y: Bool <- true;
                z: String <- "String";
            };
        "#,
        );

        assert!(SemanticAnalyzer::analyze(&program).is_ok());
    }

    #[test]
    fn constant_attributes() {
        let (s_table, program) = parse_program(
            r#"
            class Main {
                x: Int <- 1;
                y: Bool <- true;
                z: String <- "String";
            };
        "#,
        );

        let ast = match SemanticAnalyzer::analyze(&program) {
            Ok(p) => p,
            Err(e) => panic!("Failed semantic analysis: {:#?}", e),
        };

        let expected = ast::Root {
            classes: vec![ast::ClassNode {
                name: s_table.lookup("Main").unwrap(),
                parent: None,
                features: vec![
                    ast::FeatureNode::Attribute {
                        name: s_table.lookup("x").unwrap(),
                        type_dec: ReturnType::Type(INT_ID),
                        init: Some(Box::new(ast::ExprNode {
                            kind: ast::ExprKind::IntConstant(1),
                            ty: ReturnType::Type(INT_ID),
                        })),
                    },
                    ast::FeatureNode::Attribute {
                        name: s_table.lookup("y").unwrap(),
                        type_dec: ReturnType::Type(BOOL_ID),
                        init: Some(Box::new(ast::ExprNode {
                            kind: ast::ExprKind::BoolConstant(true),
                            ty: ReturnType::Type(BOOL_ID),
                        })),
                    },
                    ast::FeatureNode::Attribute {
                        name: s_table.lookup("z").unwrap(),
                        type_dec: ReturnType::Type(STRING_ID),
                        init: Some(Box::new(ast::ExprNode {
                            kind: ast::ExprKind::StringConstant(s_table.lookup("String").unwrap()),
                            ty: ReturnType::Type(STRING_ID),
                        })),
                    },
                ],
            }],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn check_errors() {
        let (_, program) = parse_program(include_str!("../examples/foobar.cl"));
        let res = SemanticAnalyzer::analyze(&program);
        match res {
            Ok(_) => return,
            Err(e) => print!("{:#?}", e),
        }
    }

    #[test_case(include_str!("../examples/arith.cl"); "arith")]
    #[test_case(include_str!("../examples/atoi.cl"); "atoi")]
    #[test_case(include_str!("../examples/book_list.cl"); "book_list")]
    #[test_case(include_str!("../examples/cells.cl"); "cells")]
    #[test_case(include_str!("../examples/complex.cl"); "complex")]
    #[test_case(include_str!("../examples/cool.cl"); "cool")]
    #[test_case(include_str!("../examples/hairyscary.cl"); "hairyscary")]
    #[test_case(include_str!("../examples/hello_world.cl"); "hello_world")]
    #[test_case(include_str!("../examples/io.cl"); "io")]
    #[test_case(include_str!("../examples/lam.cl"); "lam")]
    #[test_case(include_str!("../examples/life.cl"); "life")]
    #[test_case(include_str!("../examples/list.cl"); "list")]
    #[test_case(include_str!("../examples/new_complex.cl"); "new_complex")]
    #[test_case(include_str!("../examples/palindrome.cl"); "palindrome")]
    #[test_case(include_str!("../examples/primes.cl"); "primes")]
    #[test_case(include_str!("../examples/sort_list.cl"); "sort_list")]
    #[test_case(include_str!("../examples/foobar.cl"); "foobar")]
    #[test_case(include_str!("../examples/sum.cl"); "sum")]
    fn type_check_examples(input: &str) {
        let (_, program) = parse_program(input);
        let res = SemanticAnalyzer::analyze(&program);
        assert!(res.is_ok(), "Semantic analysis error: {:#?}", res.err());
    }
}

mod fails_type_checking {

    use coolc::{semantic_analysis::SemanticAnalyzer, utils::parse_program};
    use test_case::test_case;

    #[test_case(include_str!("../examples/semantic_errors/mismatched_types/1.cl"); "semantic_errors_mismatched_types_1")]
    fn type_check_examples(input: &str) {
        let (_, program) = parse_program(input);
        let res = SemanticAnalyzer::analyze(&program);
        assert!(res.is_err())
    }
}