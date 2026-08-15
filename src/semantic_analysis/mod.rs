#![allow(dead_code, unused_variables, unused_imports, unused)]
use core::panic;
use std::{todo, unreachable, vec};

use crate::{
    ast::{self, ExprKind, ExprNode, FeatureNode},
    parse_tree::{self, TypeName},
    semantic_analysis::{
        inheritance_tree::InheritanceTree,
        method_table::{FormalInfo, MethodInfo, MethodTable, ReturnType},
        symbol_table::SymbolTable,
    },
    string_table::{BOOL_ID, INT_ID, OBJECT_ID, STRING_ID},
};

// TODO: NEEDS BIG REFACTOR
// rethink how to propagate errors, maybe a struct field would be better
// one single return type variant (currently we have both ReturnType and parse_tree::TypeName)
// MAYBE we can just annotate the previous tree instead of creating a new one but it would be painful to match on valid and invalid every time
// if that is not an option i think a better approach would be to consume the parse tree in order to generate the ast
// actually useful error informations
// can we NOT USE UNREACHABLE ????
// Only one semantic error for type mismatch
// Limit the semantic errors and create more general ones
// SELF_TYPE will not work first time for sure!!

pub mod inheritance_tree;
pub mod method_table;
pub mod symbol_table;

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum CompOp {
    Lt,
    Le,
}

// TODO: add relevant information to the errors
#[derive(Debug, PartialEq)]
pub enum SemanticError {
    // Class related
    InheritanceCycle,
    DuplicateClass,
    NonExistentClass(usize),

    // Method Related
    RedefinedMethodInSameClass,
    WrongOverrideSignature,
    UndefinedMethod(usize, usize),

    // Attribute Related
    AttributedMismatchedTypes,

    // Expr Related
    AssignmentToSelf,
    AssignmentTypeMismatch,
    UndeclaredIdentifier,

    InvalidArithmeticOperandType,
    InvalidNegationType,
    TypeMismatch,
    WrongNumberOfArguments,

    InvalidBlockConstruct,
}

pub enum ObjKind {
    Local,
    Formal,
    Attribute,
    SelfObject,
}

pub struct ObjInfo {
    ty: ReturnType,
    kind: ObjKind,
}

impl ObjInfo {
    pub fn new(ty: ReturnType, kind: ObjKind) -> Self {
        Self { ty, kind }
    }
}

pub struct SemanticAnalyzer {
    inheritance_tree: InheritanceTree,
    method_table: MethodTable,
}

impl SemanticAnalyzer {
    pub fn analyze(program: &parse_tree::Program) -> Result<ast::Root, Vec<SemanticError>> {
        let mut err = Vec::new();
        let mut analyzer = Self {
            inheritance_tree: InheritanceTree::build(program)?,
            method_table: MethodTable::build(program)?,
        };

        if let Err(mut errors) = analyzer.check_overrides(program) {
            err.append(&mut errors);
        }

        let root = match analyzer.type_check(program) {
            Ok(root) => Some(root),
            Err(mut errors) => {
                err.append(&mut errors);
                None
            }
        };

        if !err.is_empty() {
            return Err(err);
        }

        Ok(root.expect("Something very wrong happened. Root was supposed to be Some because analyze finished with 0 semantic errors"))
    }

    fn type_check(
        &mut self,
        program: &parse_tree::Program,
    ) -> Result<ast::Root, Vec<SemanticError>> {
        let mut err = Vec::new();
        let mut classes = Vec::new();

        for class in &program.classes {
            match class {
                parse_tree::Class::Invalid => continue,
                parse_tree::Class::Valid { .. } => {
                    let mut obj_env = SymbolTable::new();
                    match self.type_check_class(class, &mut obj_env) {
                        Ok(class) => classes.push(class),
                        Err(mut errors) => err.append(&mut errors),
                    }
                }
            }
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(ast::Root::new(classes))
    }

    fn type_check_class(
        &mut self,
        class: &parse_tree::Class,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ClassNode, Vec<SemanticError>> {
        let mut err = Vec::new();
        let mut ast_features = Vec::new();

        // Open scope for current class (Oc)
        obj_env.enter_scope();

        let (class_name, parent, class_features) = match class {
            parse_tree::Class::Valid {
                name,
                parent,
                features,
            } => (*name, *parent, features),
            _ => unreachable!("Invalid classes should have been filtered out already!"),
        };

        // first pass: bind all attributes to their type, this is already done for all methods when we call method_table::build
        for feature in class_features {
            if let parse_tree::Feature::Attribute { name, type_dec, .. } = feature {
                obj_env.add_id(
                    *name,
                    ObjInfo::new(ReturnType::from(*type_dec), ObjKind::Attribute),
                );
            }
        }

        // second pass: check all attributes for the given class
        for feature in class_features {
            match feature {
                parse_tree::Feature::Attribute { .. } => {
                    match self.type_check_attribute(class_name, feature, obj_env) {
                        Ok(attribute) => ast_features.push(attribute),
                        Err(mut errors) => err.append(&mut errors),
                    }
                }
                parse_tree::Feature::Method { .. } => {
                    match self.type_check_method(class_name, feature, obj_env) {
                        Ok(method) => ast_features.push(method),
                        Err(mut errors) => err.append(&mut errors),
                    }
                }
                parse_tree::Feature::Invalid => continue,
            }
        }

        // Close the scope of the current class
        // every time we exit a class all scopes should be cleaned up
        obj_env.exit_scope();
        assert_eq!(obj_env.scopes_len(), 0);

        if !err.is_empty() {
            return Err(err);
        }

        Ok(ast::ClassNode::new(class_name, parent, ast_features))
    }

    fn type_check_method(
        &mut self,
        current_class: usize,
        method: &parse_tree::Feature,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::FeatureNode, Vec<SemanticError>> {
        let (method_name, body) = match method {
            parse_tree::Feature::Method { name, body, .. } => (name, body),
            _ => panic!("Expected method feature variant"),
        };

        let method_info = self
            .method_table
            .lookup(&self.inheritance_tree, current_class, *method_name)
            .ok_or_else(|| vec![SemanticError::UndefinedMethod(current_class, *method_name)])?;

        let declared_rt = method_info.rt().clone();
        let formals: Vec<ast::FormalNode> = method_info
            .formals()
            .iter()
            .map(|f| ast::FormalNode {
                name: f.name(),
                type_dec: f.ty(),
            })
            .collect();

        obj_env.enter_scope();

        for formal in &formals {
            obj_env.add_id(
                formal.name,
                ObjInfo::new(ReturnType::Type(formal.type_dec), ObjKind::Formal),
            );
        }

        let result = self.type_check_expr(current_class, body, obj_env);

        obj_env.exit_scope();

        let typed_body = result?;

        if !self.is_subtype(current_class, &typed_body.ty, &declared_rt) {
            return Err(vec![SemanticError::TypeMismatch]);
        }

        Ok(ast::FeatureNode::Method {
            name: *method_name,
            params: formals,
            return_type: declared_rt,
            body: Box::new(typed_body),
        })
    }

    fn type_check_attribute(
        &mut self,
        current_class: usize,
        attribute: &parse_tree::Feature,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::FeatureNode, Vec<SemanticError>> {
        let (name, type_dec, init) = match attribute {
            parse_tree::Feature::Attribute {
                name,
                type_dec,
                init,
            } => (*name, *type_dec, init),
            _ => unreachable!("Non-attribute features should have been filtered out already!"),
        };

        let typed_init = match init {
            Some(expr) => Some(Box::new(self.type_check_attribute_init(
                current_class,
                name,
                type_dec,
                expr,
                obj_env,
            )?)),
            None => None,
        };

        Ok(ast::FeatureNode::attribute(
            name,
            ReturnType::from(type_dec),
            typed_init,
        ))
    }

    fn type_check_attribute_init(
        &mut self,
        current_class: usize,
        _name: usize,
        type_dec: TypeName,
        init: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let expr = self.type_check_expr(current_class, init, obj_env)?;

        if !self.is_subtype(current_class, &expr.ty, &ReturnType::from(type_dec)) {
            return Err(vec![SemanticError::AttributedMismatchedTypes]);
        }

        Ok(expr)
    }

    fn type_check_expr(
        &mut self,
        class_id: usize,
        expr: &parse_tree::Expr,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let mut err = Vec::new();

        let expr_node: ExprNode = match expr {
            parse_tree::Expr::BoolConstant(value) => {
                ExprNode::new(ExprKind::BoolConstant(*value), ReturnType::Type(BOOL_ID))
            }
            parse_tree::Expr::IntConstant(value) => {
                ExprNode::new(ExprKind::IntConstant(*value), ReturnType::Type(INT_ID))
            }
            parse_tree::Expr::StringConstant(value) => ExprNode::new(
                ExprKind::StringConstant(*value),
                ReturnType::Type(STRING_ID),
            ),
            parse_tree::Expr::Object(name) => self.type_check_object(class_id, *name, obj_env)?,
            parse_tree::Expr::SelfExpr => ExprNode::new(ExprKind::SelfExpr, ReturnType::SelfType),
            parse_tree::Expr::Assignment { var, expr } => {
                self.type_check_expr_assignment(class_id, var, expr, obj_env)?
            }
            parse_tree::Expr::Dispatch { expr, name, args } => {
                self.type_check_expr_dispatch(class_id, expr, *name, args, obj_env)?
            }
            parse_tree::Expr::StaticDispatch {
                expr,
                type_dec,
                name,
                args,
            } => self
                .type_check_expr_static_dispatch(class_id, expr, *type_dec, *name, args, obj_env)?,
            parse_tree::Expr::SelfDispatch { name, args } => {
                self.type_check_expr_self_dispatch(class_id, *name, args, obj_env)?
            }
            parse_tree::Expr::Conditional {
                cond,
                happy_path,
                sad_path,
            } => self.type_check_expr_conditional(class_id, cond, happy_path, sad_path, obj_env)?,
            parse_tree::Expr::Loop { cond, body } => {
                self.type_check_expr_loop(class_id, cond, body, obj_env)?
            }
            parse_tree::Expr::Block(exprs) => {
                self.type_check_expr_block(class_id, exprs, obj_env)?
            }
            parse_tree::Expr::Let {
                name,
                type_dec,
                init,
                body,
            } => match init {
                Some(init) => {
                    self.type_check_let_init(class_id, *name, type_dec, init, body, obj_env)?
                }
                None => self.type_check_let_no_init(class_id, *name, type_dec, body, obj_env)?,
            },
            parse_tree::Expr::Case { cond, branches } => {
                self.type_check_expr_case(class_id, cond, branches, obj_env)?
            }
            parse_tree::Expr::New(type_name) => ExprNode::new(
                ExprKind::New(ReturnType::from(*type_name)),
                ReturnType::from(*type_name),
            ),
            parse_tree::Expr::IsVoid(expr) => {
                self.type_check_expr_is_void(class_id, expr, obj_env)?
            }
            parse_tree::Expr::Add(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Add, obj_env)?
            }
            parse_tree::Expr::Sub(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Sub, obj_env)?
            }
            parse_tree::Expr::Mul(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Mul, obj_env)?
            }
            parse_tree::Expr::Div(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Div, obj_env)?
            }
            parse_tree::Expr::Neg(expr) => self.type_check_expr_neg(class_id, expr, obj_env)?,
            parse_tree::Expr::Lt(e1, e2) => {
                self.type_check_expr_comparison(class_id, e1, e2, CompOp::Lt, obj_env)?
            }
            parse_tree::Expr::Eq(e1, e2) => self.type_check_expr_eq(class_id, e1, e2, obj_env)?,
            parse_tree::Expr::Le(e1, e2) => {
                self.type_check_expr_comparison(class_id, e1, e2, CompOp::Le, obj_env)?
            }
            parse_tree::Expr::Not(expr) => self.type_check_expr_not(class_id, expr, obj_env)?,
            parse_tree::Expr::Invalid => unreachable!("Something went terribly wrong!"),
        };

        if !err.is_empty() {
            return Err(err);
        }

        Ok(expr_node)
    }

    fn type_check_expr_loop(
        &mut self,
        current_class: usize,
        cond: &Box<parse_tree::Expr>,
        body: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let cond = self.type_check_expr(current_class, cond, obj_env)?;

        if !self.is_subtype(current_class, &cond.ty, &ReturnType::Type(BOOL_ID)) {
            return Err(vec![SemanticError::TypeMismatch]);
        }

        let body = self.type_check_expr(current_class, body, obj_env)?;

        Ok(ExprNode::new(
            ExprKind::Loop {
                cond: Box::new(cond),
                body: Box::new(body),
            },
            ReturnType::Type(OBJECT_ID),
        ))
    }

    fn type_check_expr_eq(
        &mut self,
        current_class: usize,
        e1: &Box<parse_tree::Expr>,
        e2: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let e1 = self.type_check_expr(current_class, e1, obj_env)?;
        let e2 = self.type_check_expr(current_class, e2, obj_env)?;

        let is_primitive = |ty: &ReturnType| {
            *ty == ReturnType::Type(INT_ID)
                || *ty == ReturnType::Type(STRING_ID)
                || *ty == ReturnType::Type(BOOL_ID)
        };

        if is_primitive(&e1.ty) || is_primitive(&e2.ty) {
            if e1.ty != e2.ty {
                return Err(vec![SemanticError::TypeMismatch]);
            }
        }

        Ok(ExprNode::new(
            ExprKind::Eq(Box::new(e1), Box::new(e2)),
            ReturnType::Type(BOOL_ID),
        ))
    }

    fn type_check_expr_not(
        &mut self,
        current_class: usize,
        e: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let e = self.type_check_expr(current_class, e, obj_env)?;

        if !self.is_subtype(current_class, &e.ty, &ReturnType::Type(BOOL_ID)) {
            return Err(vec![SemanticError::TypeMismatch]);
        };

        Ok(ExprNode::new(
            ExprKind::Not(Box::new(e)),
            ReturnType::Type(BOOL_ID),
        ))
    }

    fn type_check_expr_is_void(
        &mut self,
        current_class: usize,
        e: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let e = self.type_check_expr(current_class, e, obj_env)?;

        Ok(ExprNode::new(
            ExprKind::IsVoid(Box::new(e)),
            ReturnType::Type(BOOL_ID),
        ))
    }

    fn type_check_let_init(
        &mut self,
        class_id: usize,
        var_name: usize,
        type_dec: &TypeName,
        init: &Box<parse_tree::Expr>,
        body: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let declared_ty = ReturnType::from(*type_dec);

        let e1 = self.type_check_expr(class_id, init, obj_env)?;

        if !self.is_subtype(class_id, &e1.ty, &declared_ty) {
            return Err(vec![SemanticError::TypeMismatch]);
        };

        obj_env.enter_scope();
        obj_env.add_id(var_name, ObjInfo::new(declared_ty.clone(), ObjKind::Local));

        let e2 = match self.type_check_expr(class_id, body, obj_env) {
            Ok(node) => {
                obj_env.exit_scope();
                node
            }
            Err(e) => {
                obj_env.exit_scope();
                return Err(e);
            }
        };

        let t2 = e2.ty.clone();

        Ok(ExprNode::new(
            ExprKind::Let {
                name: var_name,
                type_dec: declared_ty,
                init: Some(Box::new(e1)),
                body: Box::new(e2),
            },
            t2,
        ))
    }

    fn type_check_let_no_init(
        &mut self,
        class_id: usize,
        var_name: usize,
        type_dec: &TypeName,
        body: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let declared_ty = ReturnType::from(*type_dec);

        obj_env.enter_scope();
        obj_env.add_id(var_name, ObjInfo::new(declared_ty.clone(), ObjKind::Local));

        let e1 = match self.type_check_expr(class_id, body, obj_env) {
            Ok(node) => {
                obj_env.exit_scope();
                node
            }
            Err(e) => {
                obj_env.exit_scope();
                return Err(e);
            }
        };

        let t0 = declared_ty.clone();
        let t1 = e1.ty.clone();

        Ok(ExprNode::new(
            ExprKind::Let {
                name: var_name,
                type_dec: t0,
                init: None,
                body: Box::new(e1),
            },
            t1,
        ))
    }

    fn type_check_expr_conditional(
        &mut self,
        current_class: usize,
        predicate: &Box<parse_tree::Expr>,
        hp: &Box<parse_tree::Expr>,
        sp: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let predicate = self.type_check_expr(current_class, predicate, obj_env)?;

        if !self.is_subtype(current_class, &predicate.ty, &ReturnType::Type(BOOL_ID)) {
            return Err(vec![SemanticError::TypeMismatch]);
        }

        let hp: ExprNode = self.type_check_expr(current_class, hp, obj_env)?;
        let sp = self.type_check_expr(current_class, sp, obj_env)?;
        let rt = self.lub(current_class, &sp.ty, &hp.ty);

        Ok(ExprNode::new(
            ExprKind::Conditional {
                cond: Box::new(predicate),
                happy_path: Box::new(hp),
                sad_path: Box::new(sp),
            },
            rt,
        ))
    }

    fn type_check_expr_block(
        &mut self,
        class_id: usize,
        exprs: &Vec<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let typed_expr: Vec<ExprNode> = exprs
            .iter()
            .map(|e| self.type_check_expr(class_id, e, obj_env))
            .collect::<Result<_, _>>()?;

        let rt = typed_expr
            .last()
            .ok_or(vec![SemanticError::InvalidBlockConstruct])?
            .ty
            .clone();

        Ok(ExprNode::new(ExprKind::Block(typed_expr), rt))
    }

    fn type_check_expr_self_dispatch(
        &mut self,
        current_class: usize,
        method_name: usize,
        args: &Vec<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let typed_args: Vec<_> = args
            .iter()
            .map(|arg| self.type_check_expr(current_class, arg, obj_env))
            .collect::<Result<_, _>>()?;

        let rt = self.check_method_call(
            current_class,
            current_class,
            &ReturnType::SelfType,
            method_name,
            &typed_args,
        )?;

        Ok(ExprNode::new(
            ExprKind::SelfDispatch {
                name: method_name,
                args: typed_args,
                static_class: current_class,
            },
            rt,
        ))
    }

    fn type_check_expr_static_dispatch(
        &mut self,
        current_class: usize,
        e0: &Box<parse_tree::Expr>,
        t: usize,
        method_name: usize,
        args: &Vec<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let e0 = self.type_check_expr(current_class, e0, obj_env)?;

        if !self.is_subtype(current_class, &e0.ty, &ReturnType::Type(t)) {
            return Err(vec![SemanticError::TypeMismatch]);
        }

        let typed_args: Vec<_> = args
            .iter()
            .map(|arg| self.type_check_expr(current_class, arg, obj_env))
            .collect::<Result<_, _>>()?;

        let rt = self.check_method_call(current_class, t, &e0.ty, method_name, &typed_args)?;
        Ok(ExprNode::new(
            ExprKind::StaticDispatch {
                expr: Box::new(e0),
                type_dec: t,
                name: method_name,
                args: typed_args,
            },
            rt,
        ))
    }

    fn check_method_call(
        &mut self,
        current_class: usize,
        t0: usize,
        self_ty: &ReturnType,
        name: usize,
        args: &[ast::ExprNode],
    ) -> Result<ReturnType, Vec<SemanticError>> {
        let method_info = self
            .method_table
            .lookup(&self.inheritance_tree, t0, name)
            .ok_or_else(|| vec![SemanticError::UndefinedMethod(current_class, name)])?;

        let formals = method_info.formals();

        if formals.len() != args.len() {
            return Err(vec![SemanticError::WrongNumberOfArguments]);
        }

        for (formal, arg) in formals.iter().zip(args.iter()) {
            if !self.is_subtype(current_class, &arg.ty, &ReturnType::Type(formal.ty())) {
                return Err(vec![SemanticError::TypeMismatch]);
            }
        }

        Ok(match method_info.rt() {
            ReturnType::SelfType => self_ty.clone(),
            ty => ty.clone(),
        })
    }

    fn type_check_object(
        &mut self,
        current_class: usize,
        name: usize,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        match obj_env.lookup(&name) {
            Some(info) => Ok(ast::ExprNode::new(ExprKind::Object(name), info.ty.clone())),
            None => Err(vec![SemanticError::UndeclaredIdentifier]),
        }
    }

    fn type_check_expr_neg(
        &mut self,
        current_class: usize,
        expr: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let typed_expr = self.type_check_expr(current_class, expr, obj_env)?;
        if !self.is_subtype(current_class, &typed_expr.ty, &ReturnType::Type(INT_ID)) {
            return Err(vec![SemanticError::InvalidNegationType]);
        }

        Ok(ExprNode::new(
            ExprKind::Neg(Box::new(typed_expr)),
            ReturnType::Type(INT_ID),
        ))
    }

    fn type_check_expr_assignment(
        &mut self,
        current_class: usize,
        var: &parse_tree::Var,
        expr: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let ast_var = match var {
            parse_tree::Var::Id(id) => *id,
            parse_tree::Var::SelfValue => {
                return Err(vec![SemanticError::AssignmentToSelf]);
            }
        };

        let declared_ty = match obj_env.lookup(&ast_var) {
            Some(info) => info.ty.clone(),
            None => return Err(vec![SemanticError::UndeclaredIdentifier]),
        };

        let expr = self.type_check_expr(current_class, expr, obj_env)?;

        if !self.is_subtype(current_class, &expr.ty, &declared_ty) {
            return Err(vec![SemanticError::AssignmentTypeMismatch]);
        }

        let ty = expr.ty.clone();
        return Ok(ast::ExprNode::new(
            ast::ExprKind::Assignment {
                var: ast::Var::Id(ast_var),
                expr: Box::new(expr),
            },
            ty,
        ));
    }

    fn type_check_expr_arith(
        &mut self,
        current_class: usize,
        a: &Box<parse_tree::Expr>,
        b: &Box<parse_tree::Expr>,
        op: ArithOp,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let mut err = Vec::new();
        let a = self.type_check_expr(current_class, a, obj_env)?;
        let b = self.type_check_expr(current_class, b, obj_env)?;

        if !self.is_subtype(current_class, &a.ty, &ReturnType::Type(INT_ID)) {
            err.push(SemanticError::InvalidArithmeticOperandType);
        }
        if !self.is_subtype(current_class, &b.ty, &ReturnType::Type(INT_ID)) {
            err.push(SemanticError::InvalidArithmeticOperandType);
        }

        if !err.is_empty() {
            return Err(err);
        }

        match op {
            ArithOp::Add => Ok(ExprNode::new(
                ExprKind::Add(Box::new(a), Box::new(b)),
                ReturnType::Type(INT_ID),
            )),
            ArithOp::Sub => Ok(ExprNode::new(
                ExprKind::Sub(Box::new(a), Box::new(b)),
                ReturnType::Type(INT_ID),
            )),
            ArithOp::Mul => Ok(ExprNode::new(
                ExprKind::Mul(Box::new(a), Box::new(b)),
                ReturnType::Type(INT_ID),
            )),
            ArithOp::Div => Ok(ExprNode::new(
                ExprKind::Div(Box::new(a), Box::new(b)),
                ReturnType::Type(INT_ID),
            )),
        }
    }

    fn type_check_expr_comparison(
        &mut self,
        current_class: usize,
        e1: &Box<parse_tree::Expr>,
        e2: &Box<parse_tree::Expr>,
        op: CompOp,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ExprNode, Vec<SemanticError>> {
        let e1 = self.type_check_expr(current_class, e1, obj_env)?;

        if !self.is_subtype(current_class, &e1.ty, &ReturnType::Type(INT_ID)) {
            return Err(vec![SemanticError::TypeMismatch]);
        }

        let e2 = self.type_check_expr(current_class, e2, obj_env)?;

        if !self.is_subtype(current_class, &e2.ty, &ReturnType::Type(INT_ID)) {
            return Err(vec![SemanticError::TypeMismatch]);
        }

        match op {
            CompOp::Lt => Ok(ast::ExprNode::new(
                ExprKind::Lt(Box::new(e1), Box::new(e2)),
                ReturnType::Type(BOOL_ID),
            )),
            CompOp::Le => Ok(ast::ExprNode::new(
                ExprKind::Le(Box::new(e1), Box::new(e2)),
                ReturnType::Type(BOOL_ID),
            )),
        }
    }

    fn type_check_expr_dispatch(
        &mut self,
        current_class: usize,
        e0: &Box<parse_tree::Expr>,
        name: usize,
        args: &Vec<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let e0 = self.type_check_expr(current_class, e0, obj_env)?;

        let t0 = match &e0.ty {
            ReturnType::SelfType => current_class,
            ReturnType::Type(id) => *id,
        };

        let typed_args: Vec<_> = args
            .iter()
            .map(|arg| self.type_check_expr(current_class, arg, obj_env))
            .collect::<Result<_, _>>()?;

        let rt = self.check_method_call(current_class, t0, &e0.ty, name, &typed_args)?;

        Ok(ExprNode::new(
            ExprKind::Dispatch {
                expr: Box::new(e0),
                name,
                args: typed_args,
                static_class: t0,
            },
            rt,
        ))
    }

    fn type_check_expr_case(
        &mut self,
        current_class: usize,
        cond: &Box<parse_tree::Expr>,
        branches: &Vec<parse_tree::CaseBranch>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let cond = self.type_check_expr(current_class, cond, obj_env)?;

        let mut seen_types: Vec<usize> = Vec::new();
        let mut typed_branches: Vec<ast::CaseBranchNode> = Vec::new();
        let mut err = Vec::new();

        for branch in branches {
            if seen_types.contains(&branch.type_dec) {
                err.push(SemanticError::TypeMismatch);
                continue;
            }
            seen_types.push(branch.type_dec);

            obj_env.enter_scope();
            obj_env.add_id(
                branch.name,
                ObjInfo::new(ReturnType::Type(branch.type_dec), ObjKind::Local),
            );

            let result = self.type_check_expr(current_class, &branch.body, obj_env);

            obj_env.exit_scope();

            match result {
                Ok(typed_body) => typed_branches.push(ast::CaseBranchNode {
                    name: branch.name,
                    type_dec: branch.type_dec,
                    body: Box::new(typed_body),
                }),
                Err(mut e) => err.append(&mut e),
            }
        }

        if !err.is_empty() {
            return Err(err);
        }

        let rt = typed_branches
            .iter()
            .map(|b| b.body.ty.clone())
            .reduce(|acc, ty| self.lub(current_class, &acc, &ty))
            .ok_or_else(|| vec![SemanticError::InvalidBlockConstruct])?;

        Ok(ExprNode::new(
            ExprKind::Case {
                cond: Box::new(cond),
                branches: typed_branches,
            },
            rt,
        ))
    }

    fn check_overrides(&self, program: &parse_tree::Program) -> Result<(), Vec<SemanticError>> {
        let mut err = Vec::new();
        for class in &program.classes {
            match class {
                parse_tree::Class::Invalid => continue,
                parse_tree::Class::Valid {
                    parent, features, ..
                } => {
                    for feature in features {
                        if let parse_tree::Feature::Method {
                            name,
                            params,
                            type_dec,
                            ..
                        } = feature
                        {
                            let formal_info = params
                                .iter()
                                .map(|p| FormalInfo::new(p.name, p.type_dec))
                                .collect();

                            let return_type = match type_dec {
                                parse_tree::TypeName::SelfType => ReturnType::SelfType,
                                parse_tree::TypeName::Type(id) => ReturnType::Type(*id),
                            };

                            let base_method_info = MethodInfo::new(formal_info, return_type);

                            if let Some(p) = parent {
                                if let Some(parent_method_info) =
                                    self.method_table.lookup(&self.inheritance_tree, *p, *name)
                                {
                                    if !parent_method_info.has_same_signature(&base_method_info) {
                                        err.push(SemanticError::WrongOverrideSignature);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(())
    }

    fn is_subtype(&self, current_class: usize, sub: &ReturnType, sup: &ReturnType) -> bool {
        match (sub, sup) {
            (ReturnType::SelfType, ReturnType::SelfType) => true,
            (ReturnType::SelfType, ReturnType::Type(t)) => {
                self.inheritance_tree.is_subtype(current_class, *t)
            }
            (ReturnType::Type(_), ReturnType::SelfType) => false,
            (ReturnType::Type(s), ReturnType::Type(t)) => self.inheritance_tree.is_subtype(*s, *t),
        }
    }

    fn lub(&self, current_class: usize, a: &ReturnType, b: &ReturnType) -> ReturnType {
        let lhs = match a {
            ReturnType::SelfType => current_class,
            ReturnType::Type(id) => *id,
        };

        let rhs = match b {
            ReturnType::SelfType => current_class,
            ReturnType::Type(id) => *id,
        };

        let lub = self.inheritance_tree.lub(lhs, rhs);

        if lub == current_class
            && matches!(a, ReturnType::SelfType)
            && matches!(b, ReturnType::SelfType)
        {
            ReturnType::SelfType
        } else {
            ReturnType::Type(lub)
        }
    }
}
