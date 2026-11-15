use std::format;

use crate::{codegen::constants::HEADER_WORDS, utils::Type};

#[derive(Clone)]
pub struct AttrInfo {
    name: String,
    ty: Type,
    offset: usize,
    owner_id: usize,
}

impl AttrInfo {
    pub fn new(name: String, ty: Type, offset: usize, owner_id: usize) -> Self {
        Self {
            name,
            ty,
            offset,
            owner_id,
        }
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

#[derive(Clone)]
pub struct MethodInfo {
    name: usize,
    impl_class: usize,
    slot: usize,
}

impl MethodInfo {
    pub fn new(name: usize, impl_class: usize, slot: usize) -> Self {
        Self {
            name,
            impl_class,
            slot,
        }
    }
    pub fn name(&self) -> usize {
        self.name
    }
    pub fn impl_class(&self) -> usize {
        self.impl_class
    }
    pub fn slot(&self) -> usize {
        self.slot
    }
    pub fn set_impl_class(&mut self, c: usize) {
        self.impl_class = c;
    }
}

pub struct ClassInfo {
    id: usize,
    name: String,
    tag: usize,
    parent: Option<usize>,
    attrs: Vec<AttrInfo>,
    methods: Vec<MethodInfo>,
    size: usize,
}

impl ClassInfo {
    pub fn from(
        id: usize,
        name: String,
        tag: usize,
        parent: Option<usize>,
        attrs: Vec<AttrInfo>,
        methods: Vec<MethodInfo>,
        size: usize,
    ) -> Self {
        Self {
            id,
            name,
            tag,
            parent,
            attrs,
            methods,
            size,
        }
    }

    pub fn size(&self) -> usize {
        HEADER_WORDS + self.attrs.len()
    }

    pub fn disp_label(&self) -> String {
        format!("{}_dispTab", self.name)
    }

    pub fn proto_obj_label(&self) -> String {
        format!("{}_protObj", self.name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag(&self) -> usize {
        self.tag
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn attrs(&self) -> &Vec<AttrInfo> {
        &self.attrs
    }

    pub fn methods(&self) -> &Vec<MethodInfo> {
        &self.methods
    }
}
