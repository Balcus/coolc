mod succeeds_type_check {
    use core::panic;
use std::{assert_eq, println, vec};
    use test_case::test_case;
    use coolc::{ast, semantic_analysis::{SemanticAnalyzer, method_table::ReturnType}, utils::parse_program};
    use coolc::semantic_analysis::builtins::{BOOL_ID, INT_ID, STRING_ID};

    #[test]
    fn object_inheritance() {
        let (_, program) = parse_program(r#"
            class Main inherits Object {
                x: Int <- 1;
                y: Bool <- true;
                z: String <- "String";
            };
        "#);

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
    fn test_hairyscary() {
        let (_, program) = parse_program(include_str!("../examples/hairyscary.cl"));
        let res = SemanticAnalyzer::analyze(&program);
        match res {
            Ok(_) => {
                assert!(true);
            }
            Err(e) => {
                println!("{:#?}", e);
                assert!(false);
            }
        }
        // assert!(SemanticAnalyzer::analyze(&program).is_ok());
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
    fn type_check_examples(_: &str, input: &str) {
        let (_, program) = parse_program(input);
        assert!(SemanticAnalyzer::analyze(&program).is_ok());
    }
}
