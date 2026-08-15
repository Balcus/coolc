mod succeds_type_check {
    use std::{assert_eq, vec};

    use coolc::{
        ast,
        semantic_analysis::{SemanticAnalyzer, method_table::ReturnType},
        string_table::{BOOL_ID, INT_ID, STRING_ID},
        utils::parse_program,
    };

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
}
