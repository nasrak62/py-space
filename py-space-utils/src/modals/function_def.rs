use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FunctionDef {
    pub name: String,
    pub file: PathBuf,
    pub class_name: Option<String>,
}

impl FunctionDef {
    pub fn new(name: String, file: PathBuf, class_name: Option<String>) -> Self {
        Self {
            name,
            file,
            class_name,
        }
    }

    pub fn full_name(&self) -> String {
        match &self.class_name {
            None => self.name.clone(),
            Some(value) => String::from(value) + "." + &self.name,
        }
    }
}
