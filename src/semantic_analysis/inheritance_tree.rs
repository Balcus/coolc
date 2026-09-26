use crate::semantic_analysis::SemanticErrorKind::InheritanceCycle;
use crate::semantic_analysis::builtins::{BUILTINS, OBJECT_ID};
use crate::{
    parse_tree::{Class, Program},
    semantic_analysis::SemanticError,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct InheritanceTree {
    inner: HashMap<usize, Option<usize>>,
}

impl InheritanceTree {
    pub fn build(ast: &Program) -> Result<Self, Vec<SemanticError>> {
        let mut tree = Self {
            inner: HashMap::new(),
        };

        let mut err = Vec::new();

        tree.seed();

        for class in &ast.classes {
            if let Class::Valid { name, parent, .. } = class {
                if tree.inner.contains_key(name) {
                    err.push(SemanticError {
                        kind: super::SemanticErrorKind::DuplicateClass { name: *name },
                        span: None,
                    });
                    continue;
                }

                // Insert object in the class hierarchy
                let parent = match parent {
                    Some(parent) => Some(*parent),
                    None => Some(OBJECT_ID),
                };

                tree.inner.insert(*name, parent);
            }
        }

        for parent in tree.inner.values().filter_map(|&p| p) {
            if !tree.inner.contains_key(&parent) {
                err.push(SemanticError {
                    kind: super::SemanticErrorKind::UndefinedClass { name: parent },
                    span: None,
                });
            }
        }

        if tree.has_cycle() {
            err.push(SemanticError {
                kind: InheritanceCycle,
                span: None,
            });
        }

        if !err.is_empty() {
            return Err(err);
        }

        Ok(tree)
    }

    pub fn seed(&mut self) {
        for class in BUILTINS {
            self.inner.insert(class.id, class.parent);
        }
    }

    pub fn contains(&self, class: usize) -> bool {
        self.inner.contains_key(&class)
    }

    fn has_cycle(&self) -> bool {
        for &key in self.inner.keys() {
            let mut seen = HashSet::from([key]);
            let mut current = self.parent(key);

            while let Some(class) = current {
                if !seen.insert(class) {
                    return true;
                }

                current = self.parent(class);
            }
        }

        false
    }

    pub fn is_ancestor(&self, ancestor: usize, descendant: usize) -> bool {
        let mut current = descendant;

        while let Some(parent) = self.parent(current) {
            if parent == ancestor {
                return true;
            }

            current = parent;
        }

        false
    }

    pub fn is_subtype(&self, subtype: usize, supertype: usize) -> bool {
        subtype == supertype || self.is_ancestor(supertype, subtype)
    }

    pub fn parent(&self, class: usize) -> Option<usize> {
        self.inner.get(&class).copied().flatten()
    }

    #[allow(unreachable_code)]
    pub fn lub(&self, a: usize, b: usize) -> usize {
        let mut a_anc = HashSet::new();
        let mut current = Some(a);
        while let Some(class) = current {
            a_anc.insert(class);
            current = self.parent(class);
        }
        current = Some(b);
        while let Some(class) = current {
            if a_anc.contains(&class) {
                return class;
            }
            current = self.parent(class);
        }

        unreachable!(
            "There needs to be at least one common ancestor between any 2 classes in COOL!"
        );

        0
    }
}

#[cfg(test)]
mod test {
    use crate::semantic_analysis::builtins::{BOOL_ID, INT_ID, IO_ID, OBJECT_ID, STRING_ID};
    use crate::{
        semantic_analysis::{SemanticError, SemanticErrorKind, inheritance_tree::InheritanceTree},
        utils::parse_program,
    };

    #[test]
    fn is_seeded() {
        let (s_table, program) = parse_program(
            r#"
                class A {};
            "#,
        );

        let tree = InheritanceTree::build(&program).unwrap();
        let a = s_table.lookup("A").unwrap();

        assert_eq!(tree.parent(a), Some(OBJECT_ID));

        assert!(tree.contains(OBJECT_ID));
        assert!(tree.contains(BOOL_ID));
        assert!(tree.contains(STRING_ID));
        assert!(tree.contains(INT_ID));
        assert!(tree.contains(IO_ID));

        assert_eq!(tree.parent(BOOL_ID), Some(OBJECT_ID));
        assert_eq!(tree.parent(STRING_ID), Some(OBJECT_ID));
        assert_eq!(tree.parent(INT_ID), Some(OBJECT_ID));
        assert_eq!(tree.parent(IO_ID), Some(OBJECT_ID));
    }
    #[test]
    fn valid_hierarchy() {
        let (s_table, program) = parse_program(
            r#"
                class A {};
                class B inherits A {};
                class C inherits A {};
                class D inherits B {};
            "#,
        );

        let tree = InheritanceTree::build(&program).unwrap();

        let a = s_table.lookup("A").unwrap();
        let b = s_table.lookup("B").unwrap();
        let c = s_table.lookup("C").unwrap();
        let d = s_table.lookup("D").unwrap();

        assert_eq!(tree.parent(a), Some(OBJECT_ID));
        assert_eq!(tree.parent(b), Some(a));
        assert_eq!(tree.parent(c), Some(a));
        assert_eq!(tree.parent(d), Some(b));
    }

    #[test]
    fn circular_hierarchy() {
        let (_, program) = parse_program(
            r#"
                class A inherits B {};
                class B inherits A {};
            "#,
        );

        let errors = InheritanceTree::build(&program).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            SemanticError {
                kind: SemanticErrorKind::InheritanceCycle,
                span: None
            }
        );
    }

    #[test]
    fn nonexistent_parent() {
        let (s_table, program) = parse_program(
            r#"
                class A inherits B {};
            "#,
        );

        let b = s_table.lookup("B").unwrap();
        let errors = InheritanceTree::build(&program).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            SemanticError {
                kind: SemanticErrorKind::UndefinedClass { name: b },
                span: None
            }
        );
    }

    #[test]
    fn duplicate_class() {
        let (s_table, program) = parse_program(
            r#"
                class A {};
                class A {};
            "#,
        );

        let errors = InheritanceTree::build(&program).unwrap_err();
        let a = s_table.lookup("A").unwrap();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            SemanticError {
                kind: SemanticErrorKind::DuplicateClass { name: a },
                span: None
            }
        );
    }

    #[test]
    fn subtype_and_ancestor() {
        let (s_table, program) = parse_program(
            r#"
                class A {};
                class B inherits A {};
                class C inherits B {};
            "#,
        );

        let tree = InheritanceTree::build(&program).unwrap();

        let a = s_table.lookup("A").unwrap();
        let b = s_table.lookup("B").unwrap();
        let c = s_table.lookup("C").unwrap();

        assert!(tree.is_ancestor(a, b));
        assert!(tree.is_ancestor(a, c));
        assert!(tree.is_ancestor(b, c));

        assert!(!tree.is_ancestor(c, b));
        assert!(!tree.is_ancestor(c, a));

        assert!(tree.is_subtype(c, a));
        assert!(tree.is_subtype(c, b));
        assert!(tree.is_subtype(c, c));

        assert!(!tree.is_subtype(a, c));
    }

    #[test]
    fn lub() {
        let (s_table, program) = parse_program(
            r#"
                class A {};
                class B inherits A {};
                class C inherits A {};
                class D inherits B {};
            "#,
        );

        let tree = InheritanceTree::build(&program).unwrap();

        let a = s_table.lookup("A").unwrap();
        let b = s_table.lookup("B").unwrap();
        let c = s_table.lookup("C").unwrap();
        let d = s_table.lookup("D").unwrap();

        assert_eq!(tree.lub(d, d), d);
        assert_eq!(tree.lub(d, b), b);
        assert_eq!(tree.lub(d, c), a);
        assert_eq!(tree.lub(b, c), a);
        assert_eq!(tree.lub(c, a), a);
    }
}
