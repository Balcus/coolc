use crate::{
    ast,
    semantic_analysis::{SemanticError, inheritance_tree::InheritanceTree},
};
use std::collections::HashMap;

type ClassId = usize;
type MethodId = usize;
type TypeId = usize;
type ObjectId = usize;

#[derive(Debug, PartialEq)]
pub enum ReturnType {
    SelfType,
    Type(TypeId),
}

#[derive(Debug, PartialEq)]
pub struct MethodInfo {
    formal_info: Vec<FormalInfo>,
    return_type: ReturnType,
}

impl MethodInfo {
    pub fn new(formal_info: Vec<FormalInfo>, return_type: ReturnType) -> Self {
        Self {
            formal_info,
            return_type,
        }
    }

    pub fn has_same_signature(&self, other: &MethodInfo) -> bool {
        self.return_type == other.return_type
            && self.formal_info.len() == other.formal_info.len()
            && self
                .formal_info
                .iter()
                .zip(&other.formal_info)
                .all(|(a, b)| a.ty == b.ty)
    }
}

#[derive(Debug, PartialEq)]
pub struct FormalInfo {
    name: ObjectId,
    ty: TypeId,
}

impl FormalInfo {
    pub fn new(name: ObjectId, ty: TypeId) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
pub struct MethodTable {
    inner: HashMap<(ClassId, MethodId), MethodInfo>,
}

impl MethodTable {
    pub fn build(program: &ast::Program) -> Result<Self, Vec<SemanticError>> {
        let mut table = Self {
            inner: HashMap::new(),
        };

        let mut errors = Vec::new();

        for class in &program.classes {
            match class {
                ast::Class::Invalid => continue,

                ast::Class::Valid { name, features, .. } => {
                    let defining_class = *name;

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

                            let method_info = MethodInfo {
                                formal_info,
                                return_type,
                            };

                            if let Err(error) = table.insert(defining_class, *name, method_info) {
                                errors.push(error);
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(table)
        } else {
            Err(errors)
        }
    }

    fn insert(
        &mut self,
        class_id: ClassId,
        method_id: MethodId,
        method_info: MethodInfo,
    ) -> Result<(), SemanticError> {
        if self.inner.contains_key(&(class_id, method_id)) {
            return Err(SemanticError::RedefinedMethodInSameClass);
        }

        self.inner.insert((class_id, method_id), method_info);
        Ok(())
    }

    pub fn get(&self, class_id: ClassId, method_id: MethodId) -> Option<&MethodInfo> {
        self.inner.get(&(class_id, method_id))
    }

    pub fn contains(&self, class_id: ClassId, method_id: MethodId) -> bool {
        self.inner.contains_key(&(class_id, method_id))
    }

    pub fn lookup(
        &self,
        inheritance: &InheritanceTree,
        class_id: ClassId,
        method_id: MethodId,
    ) -> Option<&MethodInfo> {
        let mut current = Some(class_id);

        while let Some(class_id) = current {
            if let Some(method) = self.get(class_id, method_id) {
                return Some(method);
            }

            current = inheritance.parent(class_id);
        }

        None
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        semantic_analysis::method_table::{FormalInfo, MethodInfo, MethodTable, ReturnType},
        utils::parse_program,
    };

    #[test]
    fn valid_methods() {
        let (s_table, program) = parse_program(
            r#"
            class A {
                foo() : Int { 0 };
            };
            class B {
                foo() : Bool { true };
            };
            class C {
                foo(x: Int, y: Bool): Int {
                    x
                };
            };
            class D {
                foo(): SELF_TYPE { new SELF_TYPE };
            };

            class E inherits A {
                add(x: Int, y: Int): Int {
                    x + y;
                };
            };
        "#,
        );

        let tbl = MethodTable::build(&program).unwrap();

        let a = s_table.lookup("A").unwrap();
        let b = s_table.lookup("B").unwrap();
        let c = s_table.lookup("C").unwrap();
        let d = s_table.lookup("D").unwrap();
        let e = s_table.lookup("E").unwrap();

        let x = s_table.lookup("x").unwrap();
        let y = s_table.lookup("y").unwrap();

        let foo = s_table.lookup("foo").unwrap();
        let add = s_table.lookup("add").unwrap();

        let int = s_table.lookup("Int").unwrap();
        let boolean = s_table.lookup("Bool").unwrap();

        assert_eq!(
            tbl.get(a, foo),
            Some(&MethodInfo::new(vec![], ReturnType::Type(int)))
        );
        assert_eq!(
            tbl.get(b, foo),
            Some(&MethodInfo::new(vec![], ReturnType::Type(boolean)))
        );
        assert_eq!(
            tbl.get(c, foo),
            Some(&MethodInfo::new(
                vec![FormalInfo::new(x, int), FormalInfo::new(y, boolean)],
                ReturnType::Type(int)
            ))
        );
        assert_eq!(
            tbl.get(d, foo),
            Some(&MethodInfo::new(vec![], ReturnType::SelfType))
        );
        assert_eq!(
            tbl.get(e, add),
            Some(&MethodInfo::new(
                vec![FormalInfo::new(x, int), FormalInfo::new(y, int)],
                ReturnType::Type(int)
            ))
        );
        assert!(tbl.get(a, add).is_none());
        assert!(tbl.get(e, foo).is_none());
    }

    #[test]
    fn duplicate_methods_inside_class() {
        let (_, program) = parse_program(
            r#"
            class A {
                foo() : Int { 0 };
                foo() : Int { 1 };
            };
        "#,
        );

        assert!(MethodTable::build(&program).is_err())
    }
}
