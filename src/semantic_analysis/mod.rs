#![allow(dead_code, unused_variables, unused_imports, unused)]
use core::panic;
use std::collections::HashMap;
use std::{todo, unreachable, vec};

use clap::error::ErrorKind::WrongNumberOfValues;

use crate::semantic_analysis::SemanticErrorKind::{InvalidBlockConstruct, WrongOverrideSignature};
use crate::semantic_analysis::builtins::{BOOL_ID, INT_ID, OBJECT_ID, STRING_ID};
use crate::utils::Span;
use crate::{
    ast::{self, ExprKind, ExprNode, FeatureNode},
    parse_tree::{self, TypeName},
    semantic_analysis::{
        inheritance_tree::InheritanceTree,
        method_table::{FormalInfo, MethodInfo, MethodTable, ReturnType},
        symbol_table::SymbolTable,
    },
};
// TODO: NEEDS BIG REFACTOR
// rethink how to propagate errors, maybe a struct field would be better
// one single return type variant (currently we have both ReturnType and parse_tree::TypeName)
// MAYBE we can just annotate the previous tree instead of creating a new one but it would be painful to match on valid and invalid every time
// if that is not an option i think a better approach would be to consume the parse tree in order to generate the ast
// actually useful error information
// can we NOT USE UNREACHABLE ????
// Only one semantic error for type mismatch
// Limit the semantic errors and create more general ones
// SELF_TYPE will not work first time for sure!!
// Seed the environment with the base classes and methods for them:

pub mod builtins;
pub mod inheritance_tree;
pub mod method_table;
pub mod symbol_table;

#[derive(Debug)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum CompOp {
    Lt,
    Le,
}

#[derive(Debug, PartialEq)]
pub struct SemanticError {
    kind: SemanticErrorKind,
    span: Option<Span>,
}

impl SemanticError {
    pub fn new(kind: SemanticErrorKind, span: Option<Span>) -> Self {
        Self { kind, span }
    }
}

// TODO: add relevant information to the errors
#[derive(Debug, PartialEq)]
pub enum SemanticErrorKind {
    // Class related
    InheritanceCycle,
    DuplicateClass {
        name: usize,
    },
    UndefinedClass {
        name: usize,
    },

    // Method related
    RedefinedMethod {
        class: usize,
        method: usize,
    },
    WrongOverrideSignature {
        class: usize,
        method: usize,
    },
    UndefinedMethod {
        class: usize,
        method: usize,
    },

    // Attribute related
    AttributeMismatch {
        attribute: usize,
        found: ReturnType,
    },

    // Expr related
    AssignmentToSelf,
    UndeclaredIdentifier {
        name: usize,
    },

    InvalidArithmeticOperandType {
        found: ReturnType,
    },
    InvalidNegationType {
        found: ReturnType,
    },
    TypeMismatch {
        expected: ReturnType,
        found: ReturnType,
    },
    WrongNumberOfArguments {
        expected: usize,
        found: usize,
    },
    DuplicateCaseBranchType {
        ty: usize,
    },
    InvalidBlockConstruct,
}

#[derive(Debug)]
pub enum ObjKind {
    Local,
    Formal,
    Attribute,
    SelfObject,
}

#[derive(Debug)]
pub struct ObjInfo {
    ty: ReturnType,
    kind: ObjKind,
}

impl ObjInfo {
    pub fn new(ty: ReturnType, kind: ObjKind) -> Self {
        Self { ty, kind }
    }
}

#[derive(Debug)]
pub struct SemanticAnalyzer {
    inheritance_tree: InheritanceTree,
    method_table: MethodTable,
}

impl SemanticAnalyzer {
    pub fn analyze(program: &parse_tree::Program) -> Result<ast::Root, Vec<SemanticError>> {
        let mut errors = Vec::new();
        let mut analyzer = Self {
            inheritance_tree: InheritanceTree::build(program)?,
            method_table: MethodTable::build(program)?,
        };

        if let Err(e) = analyzer.check_overrides(program) {
            errors.extend(e);
        }

        match analyzer.type_check(program) {
            Ok(root) if errors.is_empty() => Ok(root),
            Ok(_) => Err(errors),
            Err(e) => {
                errors.extend(e);
                Err(errors)
            }
        }
    }

    fn type_check(
        &mut self,
        program: &parse_tree::Program,
    ) -> Result<ast::Root, Vec<SemanticError>> {
        let mut errors = Vec::new();
        let mut classes = Vec::new();

        // class id -> class, so type_check_class can walk a class's ancestors
        // and bring their attributes into scope
        let class_map: HashMap<usize, &parse_tree::Class> = program
            .classes
            .iter()
            .filter_map(|class| match class {
                parse_tree::Class::Valid { name, .. } => Some((*name, class)),
                parse_tree::Class::Invalid => None,
            })
            .collect();

        for class in &program.classes {
            match class {
                parse_tree::Class::Invalid => continue,
                parse_tree::Class::Valid { .. } => {
                    let mut obj_env = SymbolTable::new();
                    match self.type_check_class(class, &class_map, &mut obj_env) {
                        Ok(class) => classes.push(class),
                        Err(e) => errors.extend(e),
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ast::Root::new(classes))
    }

    // a class with no parent should by default have the parent Object
    fn type_check_class(
        &mut self,
        class: &parse_tree::Class,
        class_map: &HashMap<usize, &parse_tree::Class>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ClassNode, Vec<SemanticError>> {
        let mut errors = Vec::new();
        let mut ast_features = Vec::new();

        // Open scope for current class (Oc)
        obj_env.enter_scope();

        let (class_name, parent, class_features) = match class {
            parse_tree::Class::Valid {
                name,
                parent,
                features,
            } => (*name, *parent, features),
            // TODO: find out how to avoid this but later!
            _ => unreachable!("Invalid classes should have been filtered out already!"),
        };

        // Inherited attributes: a COOL class sees every attribute of all its
        // ancestors. Collect the ancestor chain, nearest parent first.
        // (Terminates: analyze() returns early if InheritanceTree::build found a cycle.)
        let mut ancestors = Vec::new();
        let mut current = self.inheritance_tree.parent(class_name);
        while let Some(ancestor) = current {
            ancestors.push(ancestor);
            current = self.inheritance_tree.parent(ancestor);
        }

        // Bind them farthest-first. Builtin ancestors (Object, IO, ...) aren't in
        // class_map, but they declare no attributes, so skipping them is correct.
        for ancestor in ancestors.into_iter().rev() {
            if let Some(parse_tree::Class::Valid { features, .. }) =
                class_map.get(&ancestor).copied()
            {
                for feature in features {
                    if let parse_tree::Feature::Attribute { name, type_dec, .. } = feature {
                        obj_env.add_id(
                            *name,
                            ObjInfo::new(ReturnType::from(*type_dec), ObjKind::Attribute),
                        );
                    }
                }
            }
        }

        // first pass: bind all of this class's own attributes to their type, this is already done for all methods when we call method_table::build
        // attributes cannot have SELF_TYPE as a type in COOL
        for feature in class_features {
            if let parse_tree::Feature::Attribute { name, type_dec, .. } = feature {
                obj_env.add_id(
                    *name,
                    ObjInfo::new(ReturnType::from(*type_dec), ObjKind::Attribute),
                );
            }
        }

        // second pass: check all attributes and methods for the given class
        for feature in class_features {
            match feature {
                parse_tree::Feature::Attribute { .. } => {
                    match self.type_check_attribute(class_name, feature, obj_env) {
                        Ok(attribute) => ast_features.push(attribute),
                        Err(e) => errors.push(e),
                    }
                }
                parse_tree::Feature::Method { .. } => {
                    match self.type_check_method(class_name, feature, obj_env) {
                        Ok(method) => ast_features.push(method),
                        Err(e) => errors.push(e),
                    }
                }
                parse_tree::Feature::Invalid => continue,
            }
        }

        // Close the scope of the current class
        // every time we exit a class all scopes should be cleaned up
        obj_env.exit_scope();
        assert_eq!(obj_env.scopes_len(), 0);

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ast::ClassNode::new(class_name, parent, ast_features))
    }

    fn type_check_method(
        &mut self,
        current_class: usize,
        method: &parse_tree::Feature,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::FeatureNode, SemanticError> {
        let (method_name, body) = match method {
            parse_tree::Feature::Method { name, body, .. } => (name, body),
            _ => panic!("Expected method feature variant"),
        };

        let method_info = self
            .method_table
            .lookup(&self.inheritance_tree, current_class, *method_name)
            .ok_or_else(|| SemanticError {
                kind: SemanticErrorKind::UndefinedMethod {
                    class: current_class,
                    method: *method_name,
                },
                span: None,
            })?;

        let declared_rt = method_info.rt().clone();
        let formals: Vec<ast::FormalNode> = method_info
            .formals()
            .iter()
            .map(|f: &FormalInfo| ast::FormalNode {
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
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: declared_rt,
                    found: typed_body.ty,
                },
                span: Some(body.span.clone()),
            });
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
    ) -> Result<ast::FeatureNode, SemanticError> {
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
        name: usize,
        type_dec: TypeName,
        init: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let expr = self.type_check_expr(current_class, init, obj_env)?;

        if !self.is_subtype(current_class, &expr.ty, &ReturnType::from(type_dec)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::AttributeMismatch {
                    attribute: name,
                    found: expr.ty,
                },
                span: None,
            });
        }

        Ok(expr)
    }

    fn type_check_expr(
        &mut self,
        class_id: usize,
        expr: &parse_tree::Expr,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let expr_node: ExprNode = match &expr.kind {
            parse_tree::ExprKind::BoolConstant(value) => {
                ExprNode::new(ExprKind::BoolConstant(*value), ReturnType::Type(BOOL_ID))
            }
            parse_tree::ExprKind::IntConstant(value) => {
                ExprNode::new(ExprKind::IntConstant(*value), ReturnType::Type(INT_ID))
            }
            parse_tree::ExprKind::StringConstant(value) => ExprNode::new(
                ExprKind::StringConstant(*value),
                ReturnType::Type(STRING_ID),
            ),
            parse_tree::ExprKind::Object(name) => {
                self.type_check_object(class_id, *name, obj_env)?
            }
            parse_tree::ExprKind::SelfExpr => {
                ExprNode::new(ExprKind::SelfExpr, ReturnType::SelfType)
            }
            parse_tree::ExprKind::Assignment { var, expr } => {
                self.type_check_expr_assignment(class_id, var, expr, obj_env)?
            }
            parse_tree::ExprKind::Dispatch { expr, name, args } => {
                self.type_check_expr_dispatch(class_id, expr, *name, args, obj_env)?
            }
            parse_tree::ExprKind::StaticDispatch {
                expr,
                type_dec,
                name,
                args,
            } => self
                .type_check_expr_static_dispatch(class_id, expr, *type_dec, *name, args, obj_env)?,
            parse_tree::ExprKind::SelfDispatch { name, args } => {
                self.type_check_expr_self_dispatch(class_id, *name, args, obj_env)?
            }
            parse_tree::ExprKind::Conditional {
                cond,
                happy_path,
                sad_path,
            } => self.type_check_expr_conditional(class_id, cond, happy_path, sad_path, obj_env)?,
            parse_tree::ExprKind::Loop { cond, body } => {
                self.type_check_expr_loop(class_id, cond, body, obj_env)?
            }
            parse_tree::ExprKind::Block(exprs) => {
                self.type_check_expr_block(class_id, exprs, obj_env)?
            }
            parse_tree::ExprKind::Let {
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
            parse_tree::ExprKind::Case { cond, branches } => {
                self.type_check_expr_case(class_id, cond, branches, obj_env)?
            }
            parse_tree::ExprKind::New(type_name) => ExprNode::new(
                ExprKind::New(ReturnType::from(*type_name)),
                ReturnType::from(*type_name),
            ),
            parse_tree::ExprKind::IsVoid(expr) => {
                self.type_check_expr_is_void(class_id, expr, obj_env)?
            }
            parse_tree::ExprKind::Add(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Add, obj_env)?
            }
            parse_tree::ExprKind::Sub(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Sub, obj_env)?
            }
            parse_tree::ExprKind::Mul(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Mul, obj_env)?
            }
            parse_tree::ExprKind::Div(a, b) => {
                self.type_check_expr_arith(class_id, a, b, ArithOp::Div, obj_env)?
            }
            parse_tree::ExprKind::Neg(expr) => self.type_check_expr_neg(class_id, expr, obj_env)?,
            parse_tree::ExprKind::Lt(e1, e2) => {
                self.type_check_expr_comparison(class_id, e1, e2, CompOp::Lt, obj_env)?
            }
            parse_tree::ExprKind::Eq(e1, e2) => {
                self.type_check_expr_eq(class_id, e1, e2, obj_env)?
            }
            parse_tree::ExprKind::Le(e1, e2) => {
                self.type_check_expr_comparison(class_id, e1, e2, CompOp::Le, obj_env)?
            }
            parse_tree::ExprKind::Not(expr) => self.type_check_expr_not(class_id, expr, obj_env)?,
            parse_tree::ExprKind::Invalid => unreachable!("Something went terribly wrong!"),
        };

        Ok(expr_node)
    }

    fn type_check_expr_loop(
        &mut self,
        current_class: usize,
        expr: &Box<parse_tree::Expr>,
        body: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let typed_cond = self.type_check_expr(current_class, expr, obj_env)?;

        if !self.is_subtype(current_class, &typed_cond.ty, &ReturnType::Type(BOOL_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(BOOL_ID),
                    found: typed_cond.ty,
                },
                span: Some(expr.span.clone()),
            });
        }

        let body = self.type_check_expr(current_class, body, obj_env)?;

        Ok(ExprNode::new(
            ExprKind::Loop {
                cond: Box::new(typed_cond),
                body: Box::new(body),
            },
            ReturnType::Type(OBJECT_ID),
        ))
    }

    fn type_check_expr_eq(
        &mut self,
        current_class: usize,
        expr1: &Box<parse_tree::Expr>,
        expr2: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let typed_expr1 = self.type_check_expr(current_class, expr1, obj_env)?;
        let typed_expr2 = self.type_check_expr(current_class, expr2, obj_env)?;

        let is_primitive = |ty: &ReturnType| {
            *ty == ReturnType::Type(INT_ID)
                || *ty == ReturnType::Type(STRING_ID)
                || *ty == ReturnType::Type(BOOL_ID)
        };

        if is_primitive(&typed_expr1.ty) || is_primitive(&typed_expr2.ty) {
            if typed_expr1.ty != typed_expr2.ty {
                return Err(SemanticError {
                    kind: SemanticErrorKind::TypeMismatch {
                        expected: typed_expr1.ty,
                        found: typed_expr2.ty,
                    },
                    span: Some(expr1.span.clone()),
                });
            }
        }

        Ok(ExprNode::new(
            ExprKind::Eq(Box::new(typed_expr1), Box::new(typed_expr2)),
            ReturnType::Type(BOOL_ID),
        ))
    }

    fn type_check_expr_not(
        &mut self,
        current_class: usize,
        expr: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let typed_expr = self.type_check_expr(current_class, expr, obj_env)?;

        if !self.is_subtype(current_class, &typed_expr.ty, &ReturnType::Type(BOOL_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(BOOL_ID),
                    found: typed_expr.ty,
                },
                span: Some(expr.span.clone()),
            });
        };

        Ok(ExprNode::new(
            ExprKind::Not(Box::new(typed_expr)),
            ReturnType::Type(BOOL_ID),
        ))
    }

    fn type_check_expr_is_void(
        &mut self,
        current_class: usize,
        e: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
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
    ) -> Result<ast::ExprNode, SemanticError> {
        let declared_ty = ReturnType::from(*type_dec);

        let e1 = self.type_check_expr(class_id, init, obj_env)?;

        if !self.is_subtype(class_id, &e1.ty, &declared_ty) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: declared_ty,
                    found: e1.ty,
                },
                span: Some(init.span.clone()),
            });
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
    ) -> Result<ast::ExprNode, SemanticError> {
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
    ) -> Result<ast::ExprNode, SemanticError> {
        let typed_predicate = self.type_check_expr(current_class, predicate, obj_env)?;

        if !self.is_subtype(
            current_class,
            &typed_predicate.ty,
            &ReturnType::Type(BOOL_ID),
        ) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(BOOL_ID),
                    found: typed_predicate.ty,
                },
                span: Some(predicate.span.clone()),
            });
        }

        let hp: ExprNode = self.type_check_expr(current_class, hp, obj_env)?;
        let sp = self.type_check_expr(current_class, sp, obj_env)?;
        let rt = self.lub(current_class, &sp.ty, &hp.ty);

        Ok(ExprNode::new(
            ExprKind::Conditional {
                cond: Box::new(typed_predicate),
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
    ) -> Result<ast::ExprNode, SemanticError> {
        let typed_expr: Vec<ExprNode> = exprs
            .iter()
            .map(|e| self.type_check_expr(class_id, e, obj_env))
            .collect::<Result<_, _>>()?;

        let rt = typed_expr
            .last()
            .ok_or(SemanticError {
                kind: InvalidBlockConstruct,
                span: None,
            })?
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
    ) -> Result<ast::ExprNode, SemanticError> {
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
        expr: &Box<parse_tree::Expr>,
        t: usize,
        method_name: usize,
        args: &Vec<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let e0 = self.type_check_expr(current_class, &expr, obj_env)?;

        if !self.is_subtype(current_class, &e0.ty, &ReturnType::Type(t)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(t),
                    found: e0.ty,
                },
                span: Some(expr.span.clone()),
            });
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
    ) -> Result<ReturnType, SemanticError> {
        let method_info = self
            .method_table
            .lookup(&self.inheritance_tree, t0, name)
            .ok_or_else(|| SemanticError {
                kind: SemanticErrorKind::UndefinedMethod {
                    class: current_class,
                    method: name,
                },
                span: None,
            })?;

        let formals = method_info.formals();

        if formals.len() != args.len() {
            return Err(SemanticError {
                kind: SemanticErrorKind::WrongNumberOfArguments {
                    expected: formals.len(),
                    found: args.len(),
                },
                span: None,
            });
        }

        for (formal, arg) in formals.iter().zip(args.iter()) {
            if !self.is_subtype(current_class, &arg.ty, &ReturnType::Type(formal.ty())) {
                return Err(SemanticError {
                    kind: SemanticErrorKind::TypeMismatch {
                        expected: ReturnType::Type(formal.ty()),
                        found: arg.ty.clone(),
                    },
                    span: None,
                });
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
    ) -> Result<ast::ExprNode, SemanticError> {
        match obj_env.lookup(&name) {
            Some(info) => Ok(ast::ExprNode::new(ExprKind::Object(name), info.ty.clone())),
            None => Err(SemanticError {
                kind: SemanticErrorKind::UndeclaredIdentifier { name },
                span: None,
            }),
        }
    }

    fn type_check_expr_neg(
        &mut self,
        current_class: usize,
        expr: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let typed_expr = self.type_check_expr(current_class, expr, obj_env)?;
        if !self.is_subtype(current_class, &typed_expr.ty, &ReturnType::Type(INT_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(INT_ID),
                    found: typed_expr.ty,
                },
                span: Some(expr.span.clone()),
            });
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
    ) -> Result<ast::ExprNode, SemanticError> {
        let ast_var = match var {
            parse_tree::Var::Id(id) => *id,
            parse_tree::Var::SelfValue => {
                return Err(SemanticError {
                    kind: SemanticErrorKind::AssignmentToSelf,
                    span: None,
                });
            }
        };

        let declared_ty = match obj_env.lookup(&ast_var) {
            Some(info) => info.ty.clone(),
            None => {
                return Err(SemanticError {
                    kind: SemanticErrorKind::UndeclaredIdentifier { name: ast_var },
                    span: Some(expr.span.clone()),
                });
            }
        };

        let typed_expr = self.type_check_expr(current_class, expr, obj_env)?;

        if !self.is_subtype(current_class, &typed_expr.ty, &declared_ty) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: declared_ty,
                    found: typed_expr.ty,
                },
                span: Some(expr.span.clone()),
            });
        }

        let ty = typed_expr.ty.clone();
        return Ok(ast::ExprNode::new(
            ast::ExprKind::Assignment {
                var: ast::Var::Id(ast_var),
                expr: Box::new(typed_expr),
            },
            ty,
        ));
    }

    fn type_check_expr_arith(
        &mut self,
        current_class: usize,
        expr1: &Box<parse_tree::Expr>,
        expr2: &Box<parse_tree::Expr>,
        op: ArithOp,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ast::ExprNode, SemanticError> {
        let a = self.type_check_expr(current_class, &expr1, obj_env)?;
        let b = self.type_check_expr(current_class, &expr2, obj_env)?;

        if !self.is_subtype(current_class, &a.ty, &ReturnType::Type(INT_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(INT_ID),
                    found: a.ty,
                },
                span: Some(expr1.span.clone()),
            });
        }
        if !self.is_subtype(current_class, &b.ty, &ReturnType::Type(INT_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(INT_ID),
                    found: b.ty,
                },
                span: Some(expr2.span.clone()),
            });
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
        expr1: &Box<parse_tree::Expr>,
        expr2: &Box<parse_tree::Expr>,
        op: CompOp,
        obj_env: &mut SymbolTable<usize, ObjInfo>,
    ) -> Result<ExprNode, SemanticError> {
        let e1 = self.type_check_expr(current_class, &expr1, obj_env)?;

        if !self.is_subtype(current_class, &e1.ty, &ReturnType::Type(INT_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(INT_ID),
                    found: e1.ty,
                },
                span: Some(expr1.span.clone()),
            });
        }

        let e2 = self.type_check_expr(current_class, expr2, obj_env)?;

        if !self.is_subtype(current_class, &e2.ty, &ReturnType::Type(INT_ID)) {
            return Err(SemanticError {
                kind: SemanticErrorKind::TypeMismatch {
                    expected: ReturnType::Type(INT_ID),
                    found: e2.ty,
                },
                span: Some(expr2.span.clone()),
            });
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
    ) -> Result<ast::ExprNode, SemanticError> {
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
    ) -> Result<ast::ExprNode, SemanticError> {
        let cond = self.type_check_expr(current_class, cond, obj_env)?;

        let mut seen_types: Vec<usize> = Vec::new();
        let mut typed_branches: Vec<ast::CaseBranchNode> = Vec::new();

        for branch in branches {
            if seen_types.contains(&branch.type_dec) {
                return Err(SemanticError {
                    kind: SemanticErrorKind::DuplicateCaseBranchType {
                        ty: branch.type_dec,
                    },
                    span: None,
                });
            }
            seen_types.push(branch.type_dec);

            obj_env.enter_scope();
            obj_env.add_id(
                branch.name,
                ObjInfo::new(ReturnType::Type(branch.type_dec), ObjKind::Local),
            );

            let res = self.type_check_expr(current_class, &branch.body, obj_env);

            obj_env.exit_scope();

            let typed_body = res?;

            typed_branches.push(ast::CaseBranchNode {
                name: branch.name,
                type_dec: branch.type_dec,
                body: Box::new(typed_body),
            })
        }

        let rt = typed_branches
            .iter()
            .map(|b| b.body.ty.clone())
            .reduce(|acc, ty| self.lub(current_class, &acc, &ty))
            .ok_or_else(|| SemanticError {
                kind: InvalidBlockConstruct,
                span: None,
            })?;

        Ok(ExprNode::new(
            ExprKind::Case {
                cond: Box::new(cond),
                branches: typed_branches,
            },
            rt,
        ))
    }

    fn check_overrides(&mut self, program: &parse_tree::Program) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();

        for class in &program.classes {
            match class {
                parse_tree::Class::Invalid => continue,
                parse_tree::Class::Valid {
                    name: class_id,
                    parent,
                    features,
                } => {
                    for feature in features {
                        if let parse_tree::Feature::Method {
                            name: method_name,
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
                                if let Some(parent_method_info) = self.method_table.lookup(
                                    &self.inheritance_tree,
                                    *p,
                                    *method_name,
                                ) {
                                    if !parent_method_info.has_same_signature(&base_method_info) {
                                        errors.push(SemanticError {
                                            kind: WrongOverrideSignature {
                                                class: *class_id,
                                                method: *method_name,
                                            },
                                            span: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        return Ok(());
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
