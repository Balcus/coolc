use std::format;

use crate::{
    builtins::{BOOL_ID, INT_ID, STRING_ID},
    codegen::constants::{EYECATCHER, HEADER_WORDS},
    utils::Type,
};

pub struct AttrInfo {
    name: String,
    ty: Type,
    offset: usize,
    owner: usize,
}

pub struct MethodInfo {
    name: usize,
    impl_class: usize,
    slot: usize,
}

pub struct ClassInfo {
    id: usize,
    name: String,
    tag: usize,
    parent: Option<usize>,
    attrs: Vec<AttrInfo>,
    methods: Vec<MethodInfo>,
}

impl ClassInfo {
    pub fn from(
        id: usize,
        name: String,
        tag: usize,
        parent: Option<usize>,
        attrs: Vec<AttrInfo>,
        methods: Vec<MethodInfo>,
    ) -> Self {
        Self {
            id,
            name,
            tag,
            parent,
            attrs,
            methods,
        }
    }

    pub fn size(&self) -> usize {
        HEADER_WORDS + self.attrs.len()
    }

    fn disp_label(&self) -> String {
        format!("{}_dispTab", self.name)
    }

    fn proto_obj_label(&self) -> String {
        format!("{}_protObj", self.name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag(&self) -> usize {
        self.tag
    }

    pub fn genproto(&self) -> String {
        let mut proto = format!(
            concat!(
                "        .word   {eyecatcher}\n",
                "{label}:\n",
                "        .word   {tag}\n",
                "        .word   {size}\n",
                "        .word   {disp}\n",
            ),
            eyecatcher = EYECATCHER,
            label = self.proto_obj_label(),
            tag = self.tag,
            size = self.size(),
            disp = self.disp_label(),
        );

        for attr in &self.attrs {
            let default = match attr.ty {
                Type::Fixed(id) if id == INT_ID => "int_const0",
                Type::Fixed(id) if id == BOOL_ID => "bool_const0",
                Type::Fixed(id) if id == STRING_ID => "str_const0",
                Type::Fixed(_) | Type::SelfType => "0",
            };
            proto.push_str(&format!("        .word   {}\n", default));
        }

        proto
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builtins::OBJECT_ID;

    // class Foo {
    //     first : Object;
    //     second : Object;
    // };
    #[test]
    fn check_codegen_class() {
        let foo = ClassInfo::from(
            5,
            String::from("Foo"),
            5,
            Some(OBJECT_ID),
            vec![
                AttrInfo {
                    name: String::from("first"),
                    ty: Type::Fixed(OBJECT_ID),
                    offset: 12,
                    owner: 5,
                },
                AttrInfo {
                    name: String::from("second"),
                    ty: Type::Fixed(OBJECT_ID),
                    offset: 16,
                    owner: 5,
                },
            ],
            Vec::new(),
        );

        let expected = concat!(
            "        .word   -1\n",
            "Foo_protObj:\n",
            "        .word   5\n",
            "        .word   5\n",
            "        .word   Foo_dispTab\n",
            "        .word   0\n",
            "        .word   0\n",
        );

        assert_eq!(foo.genproto(), expected);
    }
}
