use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
        if self.class_name.is_none() {
            return self.name.clone();
        }

        let class_name = self.class_name.clone().unwrap();

        class_name + "." + &self.name
    }
}
