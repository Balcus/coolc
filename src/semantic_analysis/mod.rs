#![allow(dead_code, unused_variables, unused_imports, unused)]
use crate::{
    ast::{self, ExprNode},
    parse_tree::{self, TypeName},
    semantic_analysis::{
        inheritance_tree::InheritanceTree,
        method_table::{FormalInfo, MethodInfo, MethodTable, ReturnType},
        symbol_table::SymbolTable,
    },
    string_table::{BOOL_ID, INT_ID, OBJECT_ID, STRING_ID},
};

type TypeId = usize;
type ObjectId = usize;
type ClassId = usize;

pub mod inheritance_tree;
pub mod method_table;
pub mod symbol_table;

pub enum Operand {
    Add,
    Sub,
    Mul,
    Div
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

    // Attribute Related
    AttributedMismatchedTypes,

    // Expr Related
    AssignmentToSelf,
    AssignmentTypeMismatch,
    UndeclaredIdentifier,

    InvalidArithmeticOperandType,
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
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
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
        current_class: ClassId,
        method: &parse_tree::Feature,
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
    ) -> Result<ast::FeatureNode, Vec<SemanticError>> {
        obj_env.enter_scope();

        // check method

        obj_env.exit_scope();

        todo!()
    }

    fn type_check_attribute(
        &mut self,
        current_class: ClassId,
        attribute: &parse_tree::Feature,
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
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
            Some(expr) => Some(Box::new(self.type_check_attrib_init(
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

    fn type_check_attrib_init(
        &mut self,
        current_class: ClassId,
        _name: ObjectId,
        type_dec: TypeName,
        init: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let expr = self.type_check_expr(current_class, init, obj_env)?;

        if !self.is_subtype(current_class, &expr.ty, &ReturnType::from(type_dec)) {
            return Err(vec![SemanticError::AttributedMismatchedTypes]);
        }

        Ok(expr)
    }

    fn type_check_expr(
        &mut self,
        class_id: ClassId,
        expr: &parse_tree::Expr,
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let mut err = Vec::new();

        match expr {
            parse_tree::Expr::BoolConstant(value) => return Ok(ExprNode::bool_const(value)),
            parse_tree::Expr::IntConstant(value) => return Ok(ExprNode::int_const(value)),
            parse_tree::Expr::StringConstant(value) => return Ok(ExprNode::string_const(value)),
            parse_tree::Expr::Object(value) => return Ok(ExprNode::obj_const(value)),
            parse_tree::Expr::SelfExpr => return Ok(ExprNode::self_expr()),
            parse_tree::Expr::Assignment { var, expr } => {
                match self.type_check_assignment(class_id, var, expr, obj_env) {
                    Ok(expr_node) => return Ok(expr_node),
                    Err(mut errors) => err.append(&mut errors),
                }
            }
            parse_tree::Expr::Dispatch { expr, name, args } => todo!(),
            parse_tree::Expr::StaticDispatch {
                expr,
                type_dec,
                name,
                args,
            } => todo!(),
            parse_tree::Expr::SelfDispatch { name, args } => todo!(),
            parse_tree::Expr::Conditional {
                cond,
                happy_path,
                sad_path,
            } => todo!(),
            parse_tree::Expr::Loop { cond, body } => todo!(),
            parse_tree::Expr::Block(exprs) => todo!(),
            parse_tree::Expr::Let {
                name,
                type_dec,
                init,
                body,
            } => todo!(),
            parse_tree::Expr::Case { cond, branches } => todo!(),
            parse_tree::Expr::New(type_name) => todo!(),
            parse_tree::Expr::IsVoid(expr) => todo!(),
            parse_tree::Expr::Add(a, b) => match self.type_check_arith(class_id, a, b, Operand::Add, obj_env) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            },
            parse_tree::Expr::Sub(expr, expr1) => todo!(),
            parse_tree::Expr::Mul(expr, expr1) => todo!(),
            parse_tree::Expr::Div(expr, expr1) => todo!(),
            parse_tree::Expr::Neg(expr) => todo!(),
            parse_tree::Expr::Lt(expr, expr1) => todo!(),
            parse_tree::Expr::Eq(expr, expr1) => todo!(),
            parse_tree::Expr::Le(expr, expr1) => todo!(),
            parse_tree::Expr::Not(expr) => todo!(),
            parse_tree::Expr::Invalid => todo!(),
        }

        if !err.is_empty() {
            return Err(err);
        }

        unreachable!("Something is very wrong!");
    }

    fn type_check_assignment(
        &mut self,
        current_class: ClassId,
        var: &parse_tree::Var,
        expr: &Box<parse_tree::Expr>,
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let mut err = Vec::new();

        let ast_var = match var {
            parse_tree::Var::Id(id) => *id,
            parse_tree::Var::SelfValue => {
                err.push(SemanticError::AssignmentToSelf);
                return Err(err);
            }
        };

        let declared_ty = match obj_env.lookup(&ast_var) {
            Some(info) => info.ty.clone(),
            None => {
                err.push(SemanticError::UndeclaredIdentifier);
                return Err(err);
            }
        };

        let expr = self.type_check_expr(current_class, expr, obj_env)?;

        if !self.is_subtype(current_class, &expr.ty, &declared_ty) {
            err.push(SemanticError::AssignmentTypeMismatch);
            return Err(err);
        }

        let ty = expr.ty.clone();
        return Ok(ast::ExprNode::new(
            ast::ExprKind::Assignment {
                var: ast::Var::Id(ast_var),
                expr: Box::new(expr),
            },
            ty.clone(),
        ));
    }

    fn type_check_arith(
        &mut self,
        current_class: ClassId,
        a: &Box<parse_tree::Expr>,
        b: &Box<parse_tree::Expr>,
        op: Operand,
        obj_env: &mut SymbolTable<ObjectId, ObjInfo>,
    ) -> Result<ast::ExprNode, Vec<SemanticError>> {
        let mut err = Vec::new();
        let a = self.type_check_expr(current_class, a, obj_env)?;
        let b = self.type_check_expr(current_class, b, obj_env)?;

        if !self.is_subtype(current_class, &a.ty, &ReturnType::Type(INT_ID)) {
            err.push(SemanticError::InvalidArithmeticOperandType); // needs adding to the enum
        }
        if !self.is_subtype(current_class, &b.ty, &ReturnType::Type(INT_ID)) {
            err.push(SemanticError::InvalidArithmeticOperandType);
        }

        if !err.is_empty() {
            return Err(err);
        }

        match op {
            Operand::Add => todo!(),
            Operand::Sub => todo!(),
            Operand::Mul => todo!(),
            Operand::Div => todo!(),
        }
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

    fn is_subtype(&self, current_class: ClassId, sub: &ReturnType, sup: &ReturnType) -> bool {
        match (sub, sup) {
            (ReturnType::SelfType, ReturnType::SelfType) => true,
            (ReturnType::SelfType, ReturnType::Type(t)) => {
                self.inheritance_tree.is_subtype(current_class, *t)
            }
            (ReturnType::Type(_), ReturnType::SelfType) => false,
            (ReturnType::Type(s), ReturnType::Type(t)) => self.inheritance_tree.is_subtype(*s, *t),
        }
    }
}
