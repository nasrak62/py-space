use crate::errors::py_space::PySpaceError;
use walkdir::WalkDir;

pub fn get_files_iterator() -> Result<WalkDir, PySpaceError> {
    let path = match std::env::current_dir() {
        Ok(value) => value,
        Err(error) => return Err(PySpaceError::CantGetCurrentPath(error.to_string())),
    };

    Ok(WalkDir::new(path))
}
