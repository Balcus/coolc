pub type Id = usize;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub classes: Vec<Class>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Class {
    Valid {
        name: Id,
        parent: Option<Id>,
        features: Vec<Feature>,
    },
    Invalid,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeName {
    SelfType,
    Type(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Var {
    Id(Id),
    SelfValue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Feature {
    Attribute {
        name: Id,
        type_dec: TypeName,
        init: Option<Box<Expr>>,
    },

    Method {
        name: Id,
        params: Vec<Formal>,
        type_dec: TypeName,
        body: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Formal {
    pub name: Id,
    pub type_dec: Id,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CaseBranch {
    pub name: Id,
    pub type_dec: Id,
    pub body: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    BoolConstant(bool),
    IntConstant(i64),
    StringConstant(Id),
    Object(Id),
    SelfExpr,

    Assignment {
        var: Var,
        expr: Box<Expr>,
    },

    Dispatch {
        expr: Box<Expr>,
        name: Id,
        args: Vec<Expr>,
    },

    StaticDispatch {
        expr: Box<Expr>,
        type_dec: Id,
        name: Id,
        args: Vec<Expr>,
    },

    SelfDispatch {
        name: Id,
        args: Vec<Expr>,
    },

    Conditional {
        cond: Box<Expr>,
        happy_path: Box<Expr>,
        sad_path: Box<Expr>,
    },

    Loop {
        cond: Box<Expr>,
        body: Box<Expr>,
    },

    Block(Vec<Expr>),

    Let {
        name: Id,
        type_dec: Id,
        init: Option<Box<Expr>>,
        body: Box<Expr>,
    },

    Case {
        cond: Box<Expr>,
        branches: Vec<CaseBranch>,
    },

    New(TypeName),

    IsVoid(Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),

    Lt(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}
