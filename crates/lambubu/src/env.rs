use crate::Term;
use std::collections::HashMap;

pub trait TermEnvironment {
    fn resolve_term(&self, name: &str) -> Option<Term>;
}

pub trait MutableTermEnvironment: TermEnvironment {
    fn add_term(&mut self, name: String, term: Term);
}

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
