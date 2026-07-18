use crate::{
    ast,
    semantic_analysis::{
        inheritance_tree::InheritanceTree,
        method_table::{FormalInfo, MethodInfo, MethodTable, ReturnType},
    },
};

type TypeId = usize;

pub mod inheritance_tree;
pub mod method_table;
pub mod symbol_table;

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
}

pub enum VarKind {
    Local,
    Formal,
    Attribute,
    SelfObject,
}

pub struct VarInfo {
    ty: TypeId,
    kind: VarKind,
}

pub struct SemanticAnalyzer {
    inheritance_tree: InheritanceTree,
    method_table: MethodTable,
}

impl SemanticAnalyzer {
    pub fn analyze(program: &ast::Program) -> Result<(), Vec<SemanticError>> {
        let analyzer = Self {
            inheritance_tree: InheritanceTree::build(program)?,
            method_table: MethodTable::build(program)?,
        };
        analyzer.check_overrides(program)?;

        Ok(())
    }

    fn check_overrides(&self, program: &ast::Program) -> Result<(), Vec<SemanticError>> {
        let mut err = Vec::new();
        for class in &program.classes {
            match class {
                ast::Class::Invalid => continue,
                ast::Class::Valid {
                    parent, features, ..
                } => {
                    for feature in features {
                        if let ast::Feature::Method {
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
                                ast::TypeName::SelfType => ReturnType::SelfType,
                                ast::TypeName::Type(id) => ReturnType::Type(*id),
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
}
