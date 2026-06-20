use coolc::{ast, grammar, lexer::LexerWrapper, s_table::StringTable};

#[test]
fn assignment_classes() {
    let mut s_table = StringTable::new();
    let input = r#"
        class Main {
            x: Int <- 10;
        };"#;

    let res = grammar::ProgramParser::new()
        .parse(LexerWrapper::new(&input, &mut s_table))
        .unwrap();

    assert_eq!(res.classes.len(), 1);

    match &res.classes[0] {
        ast::Class {
            name,
            parent,
            features,
        } => {
            assert_eq!(s_table.get(*name).unwrap(), "Main");
            assert_eq!(*parent, None);
            assert_eq!(features.len(), 1);

            match &features[0] {
                ast::Feature::Attribute {
                    name,
                    type_dec,
                    init,
                } => {
                    assert_eq!(s_table.get(*name).unwrap(), "x");
                    assert_eq!(s_table.get(*type_dec).unwrap(), "Int");

                    match init.as_ref().unwrap().as_ref() {
                        ast::Expr::IntConstant(10) => {}
                        other => panic!("expected IntConstant(10), got {:?}", other),
                    }
                }
                other => panic!("expected Attribute, got {:?}", other),
            }
        }
    }
}
