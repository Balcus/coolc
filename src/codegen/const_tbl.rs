use std::collections::BTreeSet;

use crate::{
    builtins::BUILTINS,
    semantic_analysis::ast::{self, ExprNode},
    string_table::StringTable,
};

pub struct ConstTable {
    ints: BTreeSet<i64>,
    strings: BTreeSet<usize>,
}

impl ConstTable {
    pub fn from(src: &ast::Root, s_table: &StringTable) -> Self {
        let mut tbl = Self {
            ints: BTreeSet::new(),
            strings: BTreeSet::new(),
        };

        tbl.add_int(0);
        if let Some(empty) = s_table.lookup("") {
            tbl.add_string(empty, s_table);
        }

        for builtin in BUILTINS {
            let class = ast::ClassNode::from(builtin);
            tbl.add_string(class.id, s_table);
            for feature in &class.features {
                match feature {
                    ast::FeatureNode::Attribute { init, .. } => {
                        if let Some(init) = init {
                            tbl.collect(init, s_table);
                        }
                    }
                    ast::FeatureNode::Method { body, .. } => tbl.collect(body, s_table),
                }
            }
        }

        for class in &src.classes {
            tbl.add_string(class.id, s_table);
            for feature in &class.features {
                match feature {
                    ast::FeatureNode::Attribute { init, .. } => {
                        if let Some(init) = init {
                            tbl.collect(init, s_table);
                        }
                    }
                    ast::FeatureNode::Method { body, .. } => tbl.collect(body, s_table),
                }
            }
        }

        tbl
    }

    fn add_int(&mut self, v: i64) {
        self.ints.insert(v);
    }

    fn add_string(&mut self, s_id: usize, s_table: &StringTable) {
        let s_len = s_table.string_from_id(s_id).unwrap().len() as i64;
        self.add_int(s_len);
        self.strings.insert(s_id);
    }

    pub fn ints(&self) -> &BTreeSet<i64> {
        &self.ints
    }

    pub fn strings(&self) -> &BTreeSet<usize> {
        &self.strings
    }

    fn collect(&mut self, e: &ExprNode, s_table: &StringTable) {
        match &e.kind {
            ast::ExprKind::BoolConstant(_)
            | ast::ExprKind::Object(_)
            | ast::ExprKind::New(_)
            | ast::ExprKind::SelfExpr => {}

            ast::ExprKind::Add(a, b)
            | ast::ExprKind::Sub(a, b)
            | ast::ExprKind::Div(a, b)
            | ast::ExprKind::Mul(a, b)
            | ast::ExprKind::Le(a, b)
            | ast::ExprKind::Lt(a, b)
            | ast::ExprKind::Eq(a, b) => {
                self.collect(&a, s_table);
                self.collect(&b, s_table);
            }

            ast::ExprKind::Neg(a) | ast::ExprKind::Not(a) | ast::ExprKind::IsVoid(a) => {
                self.collect(&a, s_table)
            }

            ast::ExprKind::Assignment { expr, .. } => self.collect(&expr, s_table),

            ast::ExprKind::Dispatch { expr, args, .. }
            | ast::ExprKind::StaticDispatch { expr, args, .. } => {
                self.collect(&expr, s_table);
                for arg in args {
                    self.collect(&arg, s_table);
                }
            }

            ast::ExprKind::SelfDispatch { args, .. } => {
                for arg in args {
                    self.collect(&arg, s_table);
                }
            }

            ast::ExprKind::Conditional {
                cond,
                happy_path,
                sad_path,
            } => {
                self.collect(&cond, s_table);
                self.collect(&happy_path, s_table);
                self.collect(&sad_path, s_table);
            }

            ast::ExprKind::Loop { cond, body } => {
                self.collect(&cond, s_table);
                self.collect(&body, s_table);
            }

            ast::ExprKind::Block(exprs) => {
                for expr in exprs {
                    self.collect(&expr, s_table);
                }
            }

            ast::ExprKind::Let { init, body, .. } => {
                if let Some(init) = init {
                    self.collect(&init, s_table);
                }
                self.collect(&body, s_table);
            }

            ast::ExprKind::Case { cond, branches } => {
                self.collect(&cond, s_table);
                for branch in branches {
                    self.collect(&branch.body, s_table);
                }
            }

            ast::ExprKind::IntConstant(v) => self.add_int(*v),

            ast::ExprKind::StringConstant(s_id) => self.add_string(*s_id, s_table),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builtins::{BOOL_ID, INT_ID, IO_ID, OBJECT_ID, STRING_ID};
    use crate::semantic_analysis::ast::{
        CaseBranchNode, ClassNode, ExprKind, ExprNode, FeatureNode, Root, Var,
    };
    use crate::utils::Type;

    // class Main {
    //     x : Int <- 5;
    //     main() : Object {
    //         {
    //             "hello";
    //             7 + 5;
    //         }
    //     };
    // };
    #[test]
    fn simple_check() {
        let mut s_table = StringTable::new();
        let main_id = s_table.insert(String::from("Main"));
        let x_id = s_table.insert(String::from("x"));
        let main_method = s_table.insert(String::from("main"));
        let hello = s_table.insert(String::from("hello"));
        let empty = s_table.lookup("").unwrap();

        let root = Root::new(vec![ClassNode::new(
            main_id,
            Some(OBJECT_ID),
            vec![
                FeatureNode::attribute(
                    x_id,
                    Type::Fixed(INT_ID),
                    Some(Box::new(ExprNode::new(
                        ExprKind::IntConstant(5),
                        Type::Fixed(INT_ID),
                    ))),
                ),
                FeatureNode::Method {
                    id: main_method,
                    params: Vec::new(),
                    return_type: Type::Fixed(OBJECT_ID),
                    body: Box::new(ExprNode::new(
                        ExprKind::Block(vec![
                            ExprNode::new(ExprKind::StringConstant(hello), Type::Fixed(STRING_ID)),
                            ExprNode::new(
                                ExprKind::Add(
                                    Box::new(ExprNode::new(
                                        ExprKind::IntConstant(7),
                                        Type::Fixed(INT_ID),
                                    )),
                                    Box::new(ExprNode::new(
                                        ExprKind::IntConstant(5),
                                        Type::Fixed(INT_ID),
                                    )),
                                ),
                                Type::Fixed(INT_ID),
                            ),
                        ]),
                        Type::Fixed(INT_ID),
                    )),
                },
            ],
        )]);

        let table = ConstTable::from(&root, &s_table);
        let expected_ints: BTreeSet<i64> = [0, 2, 3, 4, 5, 6, 7].into_iter().collect();
        assert_eq!(table.ints, expected_ints);

        let expected_strings: BTreeSet<usize> = [
            empty, main_id, hello, OBJECT_ID, IO_ID, INT_ID, BOOL_ID, STRING_ID,
        ]
        .into_iter()
        .collect();
        assert_eq!(table.strings, expected_strings);
    }

    // class Main {
    //     main() : Object {
    //         {
    //             10 + 11;
    //             12 - 13;
    //             14 * 15;
    //             16 / 17;
    //             ~18;
    //             19 < 20;
    //             21 <= 22;
    //             23 = 24;
    //             not isvoid 27;
    //             x <- 28;
    //             29.foo(30, "s1");
    //             31@Object.foo(32);
    //             foo(33, "s2");
    //             if 34 then 35 else 36 fi;
    //             while 37 loop 38 pool;
    //             let x : Int <- 39 in 40;
    //             let x : Int in 41;
    //             case 42 of y : Int => 43; z : String => "s3"; esac;
    //             true;
    //             x;
    //             self;
    //             new Main;
    //         }
    //     };
    // };
    #[test]
    fn collects_constants_from_every_expression_kind() {
        let mut s_table = StringTable::new();
        let main_id = s_table.insert(String::from("Main"));
        let main_method = s_table.insert(String::from("main"));
        let x_id = s_table.insert(String::from("x"));
        let y_id = s_table.insert(String::from("y"));
        let z_id = s_table.insert(String::from("z"));
        let foo_id = s_table.insert(String::from("foo"));
        let s1 = s_table.insert(String::from("s1"));
        let s2 = s_table.insert(String::from("s2"));
        let s3 = s_table.insert(String::from("s3"));
        let empty = s_table.lookup("").unwrap();

        let root = Root::new(vec![ClassNode::new(
            main_id,
            Some(OBJECT_ID),
            vec![FeatureNode::Method {
                id: main_method,
                params: Vec::new(),
                return_type: Type::Fixed(OBJECT_ID),
                body: Box::new(ExprNode::new(
                    ExprKind::Block(vec![
                        ExprNode::new(
                            ExprKind::Add(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(10),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(11),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Sub(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(12),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(13),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Mul(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(14),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(15),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Div(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(16),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(17),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Neg(Box::new(ExprNode::new(
                                ExprKind::IntConstant(18),
                                Type::Fixed(INT_ID),
                            ))),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Lt(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(19),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(20),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Le(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(21),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(22),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Eq(
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(23),
                                    Type::Fixed(INT_ID),
                                )),
                                Box::new(ExprNode::new(
                                    ExprKind::IntConstant(24),
                                    Type::Fixed(INT_ID),
                                )),
                            ),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Not(Box::new(ExprNode::new(
                                ExprKind::IsVoid(Box::new(ExprNode::new(
                                    ExprKind::IntConstant(27),
                                    Type::Fixed(INT_ID),
                                ))),
                                Type::Fixed(INT_ID),
                            ))),
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Assignment {
                                var: Var::Id(x_id),
                                expr: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(28),
                                    Type::Fixed(INT_ID),
                                )),
                            },
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Dispatch {
                                expr: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(29),
                                    Type::Fixed(INT_ID),
                                )),
                                name: foo_id,
                                args: vec![
                                    ExprNode::new(ExprKind::IntConstant(30), Type::Fixed(INT_ID)),
                                    ExprNode::new(
                                        ExprKind::StringConstant(s1),
                                        Type::Fixed(STRING_ID),
                                    ),
                                ],
                                static_class: OBJECT_ID,
                            },
                            Type::Fixed(OBJECT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::StaticDispatch {
                                expr: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(31),
                                    Type::Fixed(INT_ID),
                                )),
                                type_dec: OBJECT_ID,
                                name: foo_id,
                                args: vec![ExprNode::new(
                                    ExprKind::IntConstant(32),
                                    Type::Fixed(INT_ID),
                                )],
                            },
                            Type::Fixed(OBJECT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::SelfDispatch {
                                name: foo_id,
                                args: vec![
                                    ExprNode::new(ExprKind::IntConstant(33), Type::Fixed(INT_ID)),
                                    ExprNode::new(
                                        ExprKind::StringConstant(s2),
                                        Type::Fixed(STRING_ID),
                                    ),
                                ],
                                static_class: main_id,
                            },
                            Type::Fixed(OBJECT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Conditional {
                                cond: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(34),
                                    Type::Fixed(INT_ID),
                                )),
                                happy_path: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(35),
                                    Type::Fixed(INT_ID),
                                )),
                                sad_path: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(36),
                                    Type::Fixed(INT_ID),
                                )),
                            },
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Loop {
                                cond: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(37),
                                    Type::Fixed(INT_ID),
                                )),
                                body: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(38),
                                    Type::Fixed(INT_ID),
                                )),
                            },
                            Type::Fixed(OBJECT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Let {
                                name: x_id,
                                type_dec: Type::Fixed(INT_ID),
                                init: Some(Box::new(ExprNode::new(
                                    ExprKind::IntConstant(39),
                                    Type::Fixed(INT_ID),
                                ))),
                                body: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(40),
                                    Type::Fixed(INT_ID),
                                )),
                            },
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Let {
                                name: x_id,
                                type_dec: Type::Fixed(INT_ID),
                                init: None,
                                body: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(41),
                                    Type::Fixed(INT_ID),
                                )),
                            },
                            Type::Fixed(INT_ID),
                        ),
                        ExprNode::new(
                            ExprKind::Case {
                                cond: Box::new(ExprNode::new(
                                    ExprKind::IntConstant(42),
                                    Type::Fixed(INT_ID),
                                )),
                                branches: vec![
                                    CaseBranchNode {
                                        name: y_id,
                                        type_dec: INT_ID,
                                        body: Box::new(ExprNode::new(
                                            ExprKind::IntConstant(43),
                                            Type::Fixed(INT_ID),
                                        )),
                                    },
                                    CaseBranchNode {
                                        name: z_id,
                                        type_dec: STRING_ID,
                                        body: Box::new(ExprNode::new(
                                            ExprKind::StringConstant(s3),
                                            Type::Fixed(STRING_ID),
                                        )),
                                    },
                                ],
                            },
                            Type::Fixed(OBJECT_ID),
                        ),
                        ExprNode::new(ExprKind::BoolConstant(true), Type::Fixed(OBJECT_ID)),
                        ExprNode::new(ExprKind::Object(x_id), Type::Fixed(INT_ID)),
                        ExprNode::new(ExprKind::SelfExpr, Type::SelfType),
                        ExprNode::new(ExprKind::New(Type::Fixed(main_id)), Type::Fixed(main_id)),
                    ]),
                    Type::Fixed(main_id),
                )),
            }],
        )]);

        let table = ConstTable::from(&root, &s_table);
        let expected_ints: BTreeSet<i64> = [0, 2, 3, 4, 6]
            .into_iter()
            .chain(10..=24)
            .chain(27..=43)
            .collect();
        assert_eq!(table.ints, expected_ints);

        let expected_strings: BTreeSet<usize> = [
            empty, main_id, s1, s2, s3, OBJECT_ID, IO_ID, INT_ID, BOOL_ID, STRING_ID,
        ]
        .into_iter()
        .collect();
        assert_eq!(table.strings, expected_strings);
    }
}
