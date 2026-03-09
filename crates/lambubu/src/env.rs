//! This module contains macro environments
use crate::Term;
use std::collections::HashMap;

pub trait TermEnvironment {
    fn resolve_term(&self, name: &str) -> Option<Term>;
}

pub trait MutableTermEnvironment: TermEnvironment {
    fn add_term(&mut self, name: String, term: Term);
}

/// A simple environment which supports adding terms and retrieving
/// them by names.
#[derive(Clone, Debug, Default)]
pub struct RegistryEnvironment {
    terms: HashMap<String, Term>,
}

impl RegistryEnvironment {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TermEnvironment for RegistryEnvironment {
    fn resolve_term(&self, name: &str) -> Option<Term> {
        self.terms.get(name).cloned()
    }
}

impl MutableTermEnvironment for RegistryEnvironment {
    fn add_term(&mut self, name: String, term: Term) {
        self.terms.insert(name, term);
    }
}

/// Environment which aggregates multiple environments and
/// maintains its own registry.
///
/// Macros are searched in the following order:
///
/// 1. In internal registry, which can be accessed using the [crate::CompoundEnvironment::add_term]
///    method.
/// 2. In nested environments in order of their appearance.
#[derive(Default)]
pub struct CompoundEnvironment {
    envs: Vec<Box<dyn TermEnvironment>>,
    terms: HashMap<String, Term>,
}

impl MutableTermEnvironment for CompoundEnvironment {
    fn add_term(&mut self, name: String, term: Term) {
        self.terms.insert(name, term);
    }
}

impl CompoundEnvironment {
    pub fn new(envs: Vec<Box<dyn TermEnvironment>>) -> Self {
        Self {
            envs,
            terms: HashMap::new(),
        }
    }

    pub fn decompose(self) -> Vec<Box<dyn TermEnvironment>> {
        self.envs
    }
}

impl TermEnvironment for CompoundEnvironment {
    fn resolve_term(&self, name: &str) -> Option<Term> {
        self.terms
            .get(name)
            .cloned()
            .or_else(|| self.envs.iter().find_map(|x| x.resolve_term(name)))
    }
}
