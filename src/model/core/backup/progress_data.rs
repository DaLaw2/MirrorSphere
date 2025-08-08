use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::model::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressData {
    pub current_level: Vec<PathBuf>,
    pub errors: Vec<Error>,
}

impl ProgressData {
    pub fn new(current_level: Vec<PathBuf>, errors: Vec<Error>) -> ProgressData {
        ProgressData {
            current_level,
            errors
        }
    }
}
