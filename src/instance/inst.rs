// inst.rs 
// author: akrm al-hakimi
// instance structure 

use std::collections::HashMap;
use crate::class::LoxClass;

#[derive(Debug, Clone)]
pub struct LoxInstance {
    klass: LoxClass,
}

impl LoxInstance {

    pub fn new(klass: LoxClass) -> Self {
        Self {
            klass,
        }
    }
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.klass)
    }
}
