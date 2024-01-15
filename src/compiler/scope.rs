#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Local {
    pub name: String,
    pub depth: i32,
    pub is_captured: bool,
}

impl Local {
    pub fn new(name: impl Into<String>, depth: i32) -> Self {
        Self {
            name: name.into(),
            depth,
            is_captured: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

impl Upvalue {
    pub fn new(index: usize, is_local: bool) -> Self {
        Self { index, is_local }
    }
}
