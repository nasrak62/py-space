use std::{collections::HashMap, path::PathBuf};

use super::imports::Imports;

pub struct FileData {
    pub path: PathBuf,
    pub imports: HashMap<String, Vec<Imports>>,
}

impl FileData {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            imports: HashMap::new(),
        }
    }
}
