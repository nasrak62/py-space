#[derive(Clone)]
pub struct Imports {
    pub path: String,
    pub name: String,
    pub alias: Option<String>,
}

impl Imports {
    pub fn new(path: String, name: String, alias: Option<String>) -> Self {
        Self { path, name, alias }
    }
}
