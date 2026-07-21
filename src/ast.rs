use crate::{
    semantic_analysis::method_table::ReturnType,
    string_table::{BOOL_ID, INT_ID, STRING_ID},
};
use std::usize;

type Id = usize;
type ClassId = usize;

#[derive(Debug, Clone)]
pub struct Root {
    pub classes: Vec<ClassNode>,
}

impl Root {
    pub fn new(classes: Vec<ClassNode>) -> Self {
        Self { classes }
    }
}

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub name: ClassId,
    pub parent: Option<ClassId>,
    pub features: Vec<FeatureNode>,
}

impl ClassNode {
    pub fn new(name: ClassId, parent: Option<ClassId>, features: Vec<FeatureNode>) -> Self {
        Self {
            name,
            parent,
            features,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FeatureNode {
    Attribute {
        name: Id,
        type_dec: ReturnType,
        init: Option<Box<ExprNode>>,
    },
    Method {
        name: Id,
        params: Vec<FormalNode>,
        return_type: ReturnType,
        body: Box<ExprNode>,
    },
}

impl FeatureNode {
    pub fn attribute(name: Id, type_dec: ReturnType, init: Option<Box<ExprNode>>) -> Self {
        Self::Attribute {
            name,
            type_dec,
            init,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormalNode {
    pub name: Id,
    pub type_dec: ClassId,
}

#[derive(Debug, Clone)]
pub struct ExprNode {
    pub kind: ExprKind,
    pub ty: ReturnType,
}

impl ExprNode {
    pub fn new(kind: ExprKind, ty: ReturnType) -> Self {
        Self { kind, ty }
    }

    pub fn bool_const(value: &bool) -> Self {
        return ExprNode::new(ExprKind::BoolConstant(*value), ReturnType::Type(BOOL_ID));
    }

    pub fn int_const(value: &i64) -> Self {
        return ExprNode::new(ExprKind::IntConstant(*value), ReturnType::Type(INT_ID));
    }

    pub fn string_const(value: &usize) -> Self {
        return ExprNode::new(
            ExprKind::StringConstant(*value),
            ReturnType::Type(STRING_ID),
        );
    }

    pub fn self_expr() -> Self {
        return ExprNode::new(ExprKind::SelfExpr, ReturnType::SelfType);
    }

    pub fn conditional(predicate: ExprNode, sp: ExprNode, hp: ExprNode, rt: ReturnType) -> Self {
        return ExprNode::new(
            ExprKind::Conditional {
                cond: Box::new(predicate),
                happy_path: Box::new(hp),
                sad_path: Box::new(sp),
            },
            rt,
        );
    }

    pub fn dispatch(
        e0: ExprNode,
        method_name: Id,
        args: Vec<ExprNode>,
        static_class: ClassId,
        rt: ReturnType,
    ) -> Self {
        Self {
            kind: ExprKind::Dispatch {
                expr: Box::new(e0),
                name: method_name,
                args,
                static_class,
            },
            ty: rt,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    BoolConstant(bool),
    IntConstant(i64),
    StringConstant(Id),
    Object(Id),
    SelfExpr,

    Assignment {
        var: Var,
        expr: Box<ExprNode>,
    },

    Dispatch {
        expr: Box<ExprNode>,
        name: Id,
        args: Vec<ExprNode>,
        static_class: ClassId,
    },

    StaticDispatch {
        expr: Box<ExprNode>,
        type_dec: Id,
        name: Id,
        args: Vec<ExprNode>,
    },

    SelfDispatch {
        name: Id,
        args: Vec<ExprNode>,
        static_class: ClassId,
    },

    Conditional {
        cond: Box<ExprNode>,
        happy_path: Box<ExprNode>,
        sad_path: Box<ExprNode>,
    },

    Loop {
        cond: Box<ExprNode>,
        body: Box<ExprNode>,
    },

    Block(Vec<ExprNode>),

    Let {
        name: Id,
        type_dec: Id,
        init: Option<Box<ExprNode>>,
        body: Box<ExprNode>,
    },

    Case {
        cond: Box<ExprNode>,
        branches: Vec<CaseBranchNode>,
    },

    New(ReturnType),

    IsVoid(Box<ExprNode>),

    Add(Box<ExprNode>, Box<ExprNode>),
    Sub(Box<ExprNode>, Box<ExprNode>),
    Mul(Box<ExprNode>, Box<ExprNode>),
    Div(Box<ExprNode>, Box<ExprNode>),
    Neg(Box<ExprNode>),

    Lt(Box<ExprNode>, Box<ExprNode>),
    Eq(Box<ExprNode>, Box<ExprNode>),
    Le(Box<ExprNode>, Box<ExprNode>),
    Not(Box<ExprNode>),
}

#[derive(Debug, Clone)]
pub enum Var {
    Id(Id),
    SelfValue,
}

#[derive(Debug, Clone)]
pub struct CaseBranchNode {
    pub name: Id,
    pub type_dec: Id,
    pub body: Box<ExprNode>,
}
