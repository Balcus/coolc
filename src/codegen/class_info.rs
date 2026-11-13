use std::format;

use crate::{
    codegen::{
        TagGen,
        constants::{EYECATCHER, disptab_label, object_size_words, protobj_label},
    },
    semantic_analysis::ast,
    string_table::StringTable,
};

// Produces a skeleton object of type class
// The produced object must reside in the data area
pub fn codegen_class(class: ast::ClassNode, tag_gen: &mut TagGen, s_table: &StringTable) -> String {
    let class_name = s_table.string_from_id(class.id).unwrap();
    let tag = tag_gen.get();
    let label = protobj_label(class_name);
    let disp = disptab_label(class_name);
    let size = object_size_words(class.nattr());

    let mut res = format!(
        concat!(
            "        .word   {eyecatcher}\n",
            "{label}:\n",
            "        .word   {tag}\n",
            "        .word   {size}\n",
            "        .word   {disp}\n",
        ),
        eyecatcher = EYECATCHER,
        label = label,
        tag = tag,
        size = size,
        disp = disp,
    );

    for _ in 0..class.nattr() {
        res.push_str("        .word   0\n");
    }

    res
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::ReturnType;

    #[test]
    fn check_codegen_class() {
        let mut s_table = StringTable::new();
        let foo = s_table.insert(String::from("Foo"));
        let first = s_table.insert(String::from("first"));
        let second = s_table.insert(String::from("second"));

        let object_ty = ReturnType::Type(crate::semantic_analysis::builtins::OBJECT_ID);

        let class = ast::ClassNode::new(
            foo,
            None,
            vec![
                ast::FeatureNode::attribute(first, object_ty.clone(), None),
                ast::FeatureNode::attribute(second, object_ty, None),
            ],
        );

        let mut tag_gen = TagGen::new();
        for _ in 0..5 {
            tag_gen.get();
        }

        let out = codegen_class(class, &mut tag_gen, &s_table);

        let expected = concat!(
            "        .word   -1\n",
            "Foo_protObj:\n",
            "        .word   5\n",
            "        .word   5\n",
            "        .word   Foo_dispTab\n",
            "        .word   0\n",
            "        .word   0\n",
        );
        assert_eq!(out, expected);
    }
}
