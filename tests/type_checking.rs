mod succeeds_type_check {
    use coolc::builtins::{BOOL_ID, INT_ID, STRING_ID};
    use coolc::inheritance_tree::InheritanceTree;
    use coolc::semantic_analysis::ast;
    use coolc::semantic_analysis::method_table::MethodTable;
    use coolc::utils::Type;
    use coolc::{semantic_analysis::SemanticAnalyzer, utils::parse_program};
    use core::panic;
    use std::{assert_eq, vec};
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

        let i = InheritanceTree::build(&program).unwrap();
        let m = MethodTable::build(&program).unwrap();
        assert!(SemanticAnalyzer::analyze(program, &i, &m).is_ok());
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

        let i = InheritanceTree::build(&program).unwrap();
        let m = MethodTable::build(&program).unwrap();

        let ast = match SemanticAnalyzer::analyze(program, &i, &m) {
            Ok(p) => p,
            Err(e) => panic!("Failed semantic analysis: {:#?}", e),
        };

        let expected = ast::Root {
            classes: vec![ast::ClassNode {
                id: s_table.lookup("Main").unwrap(),
                parent: None,
                features: vec![
                    ast::FeatureNode::Attribute {
                        name: s_table.lookup("x").unwrap(),
                        type_dec: Type::Fixed(INT_ID),
                        init: Some(Box::new(ast::ExprNode {
                            kind: ast::ExprKind::IntConstant(1),
                            ty: Type::Fixed(INT_ID),
                        })),
                    },
                    ast::FeatureNode::Attribute {
                        name: s_table.lookup("y").unwrap(),
                        type_dec: Type::Fixed(BOOL_ID),
                        init: Some(Box::new(ast::ExprNode {
                            kind: ast::ExprKind::BoolConstant(true),
                            ty: Type::Fixed(BOOL_ID),
                        })),
                    },
                    ast::FeatureNode::Attribute {
                        name: s_table.lookup("z").unwrap(),
                        type_dec: Type::Fixed(STRING_ID),
                        init: Some(Box::new(ast::ExprNode {
                            kind: ast::ExprKind::StringConstant(s_table.lookup("String").unwrap()),
                            ty: Type::Fixed(STRING_ID),
                        })),
                    },
                ],
            }],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn inherited_attributes() {
        let (s_table, program) = parse_program(
            r#"
        class A {
            x: Int <- 1;
        };

        class B inherits A {
            y: Bool <- true;
        };

        class Main inherits B {
            z: String <- "hello";
        };
        "#,
        );

        let i = InheritanceTree::build(&program).unwrap();
        let m = MethodTable::build(&program).unwrap();

        let actual = match SemanticAnalyzer::analyze(program, &i, &m) {
            Ok(ast) => ast,
            Err(e) => panic!("Failed semantic analysis: {:#?}", e),
        };

        let class_id = s_table.lookup("Main").unwrap();
        let x_id = s_table.lookup("x").unwrap();
        let y_id = s_table.lookup("y").unwrap();
        let z_id = s_table.lookup("z").unwrap();

        let main_class = actual
            .classes
            .iter()
            .find(|class| class.id == class_id)
            .expect("Main class should exist in AST");

        let expected = vec![
            ast::FeatureNode::Attribute {
                name: x_id,
                type_dec: Type::Fixed(INT_ID),
                init: Some(Box::new(ast::ExprNode {
                    kind: ast::ExprKind::IntConstant(1),
                    ty: Type::Fixed(INT_ID),
                })),
            },
            ast::FeatureNode::Attribute {
                name: y_id,
                type_dec: Type::Fixed(BOOL_ID),
                init: Some(Box::new(ast::ExprNode {
                    kind: ast::ExprKind::BoolConstant(true),
                    ty: Type::Fixed(BOOL_ID),
                })),
            },
            ast::FeatureNode::Attribute {
                name: z_id,
                type_dec: Type::Fixed(STRING_ID),
                init: Some(Box::new(ast::ExprNode {
                    kind: ast::ExprKind::StringConstant(s_table.lookup("hello").unwrap()),
                    ty: Type::Fixed(STRING_ID),
                })),
            },
        ];

        assert_eq!(
            main_class.features, expected,
            "Main should contain its own attributes and all inherited attributes"
        );
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
    #[test_case(include_str!("../examples/self_type.cl"); "self_type")]
    #[test_case(include_str!("../examples/inheritance.cl"); "inheritance")]
    #[test_case(include_str!("../examples/activations.cl"); "activations")]
    #[test_case(include_str!("../examples/simple.cl"); "simple")]
    fn type_check_examples(input: &str) {
        let (s_table, program) = parse_program(input);
        let i = InheritanceTree::build(&program).unwrap();
        let m = MethodTable::build(&program).unwrap();
        let res = SemanticAnalyzer::analyze(program, &i, &m);
        assert!(
            res.is_ok(),
            "Semantic analysis error: {:#?}. String Table: {:#?}",
            res.err(),
            s_table
        );
    }
}

mod fails_type_checking {

    use coolc::{
        inheritance_tree::InheritanceTree,
        semantic_analysis::{SemanticAnalyzer, method_table::MethodTable},
        utils::parse_program,
    };
    use test_case::test_case;

    #[test_case(include_str!("../examples/semantic_errors/1.cl"); "semantic_errors_mismatched_types_1")]
    #[test_case(include_str!("../examples/semantic_errors/2.cl"); "semantic_errors_mismatched_types_2")]
    #[test_case(include_str!("../examples/semantic_errors/3.cl"); "semantic_errors_mismatched_types_3")]
    #[test_case(include_str!("../examples/semantic_errors/4.cl"); "semantic_errors_mismatched_types_4")]
    fn type_check_examples(input: &str) {
        let (_, program) = parse_program(input);
        let i = InheritanceTree::build(&program).unwrap();
        let m = MethodTable::build(&program).unwrap();
        let res = SemanticAnalyzer::analyze(program, &i, &m);
        assert!(res.is_err())
    }
}
