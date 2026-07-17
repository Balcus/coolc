use std::collections::{HashMap, HashSet};
use crate::{ast::{Class, Program}, semantic_analysis::SemanticError};

#[derive(Debug)]
struct InheritanceTree<'a> {
    ast: &'a Program,
    pub inner: HashMap<usize, Option<usize>>
}

impl<'a> InheritanceTree<'a> {
    pub fn from(ast: &'a Program) -> Self {
        Self {
            ast,
            inner: HashMap::new()
        }
    }

    pub fn build(mut self) -> Result<Self, Vec<SemanticError>> {
        let mut err = Vec::new();

        for class in &self.ast.classes {
            if let Class::Valid {name, parent, ..} = class {
                if self.inner.contains_key(&name) {
                    err.push(SemanticError::DuplicateClass);
                    continue;
                }

                self.inner.insert(*name, *parent);
            }
        }

        // Check for missing classes in the class hierarchy
       for parent in self.inner.values().filter_map(|&p| p) {
            if !self.inner.contains_key(&parent) {
                err.push(SemanticError::NonExistentClass(parent));
            }
        } 

        if self.has_cycle() {
            err.push(SemanticError::InheritanceCycle);
        }
        
        if !err.is_empty() {
            return Err(err)
        }
        
        Ok(self)
    }

    fn has_cycle(&self) -> bool {
        for &key in self.inner.keys() {
            let mut seen = HashSet::from([key]);
            let mut current = self.inner[&key];

            while let Some(class) = current {
                if !seen.insert(class) {
                    return true;
                }

                current = match self.inner.get(&class) {
                    Some(parent) => *parent,
                    None => break
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::LexerWrapper, parser, s_table::StringTable, semantic_analysis::{SemanticError, inheritance_tree::InheritanceTree}};

    #[test]
    fn valid_hierarchy() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
            class A {};
            class B inherits A {};
            class C inherits A {};
            class D inherits B {};
        "#;
        let tokens = Box::new(LexerWrapper::new(input, &mut s_table, "".to_string()));
        let mut parser = parser::Parser::new(&mut errors);

        let program = parser.parse(tokens).unwrap();
        let inheritance_tree = InheritanceTree::from(&program).build().unwrap();

        let a = s_table.lookup("A").unwrap();
        let b = s_table.lookup("B").unwrap();
        let c = s_table.lookup("C").unwrap();
        let d = s_table.lookup("D").unwrap();

        assert_eq!(inheritance_tree.inner.get(&a), Some(&None));
        assert_eq!(inheritance_tree.inner.get(&b), Some(&Some(a)));
        assert_eq!(inheritance_tree.inner.get(&c), Some(&Some(a)));
        assert_eq!(inheritance_tree.inner.get(&d), Some(&Some(b)));
    }

    #[test]
    fn cycle_hierarchy() {
        let mut s_table = StringTable::new();
        let mut errors = Vec::new();
        let input = r#"
            class A inherits B {};
            class B inherits A {};
        "#;
        let tokens = Box::new(LexerWrapper::new(input, &mut s_table, "".to_string()));
        let mut parser = parser::Parser::new(&mut errors);

        let program = parser.parse(tokens).unwrap();
        let inheritance_tree = InheritanceTree::from(&program).build();

        assert!(inheritance_tree.is_err());

        let errors = inheritance_tree.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], SemanticError::InheritanceCycle);
    }
}