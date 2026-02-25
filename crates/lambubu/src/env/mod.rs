use crate::Term;
use std::collections::HashMap;

pub mod church;
pub use church::ChurchEnvironment;

pub trait TermEnvironment {
    fn resovle_term(&self, name: &str) -> Option<Term>;
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
    fn resovle_term(&self, name: &str) -> Option<Term> {
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
    fn resovle_term(&self, name: &str) -> Option<Term> {
        self.terms.get(name).cloned().or(self
            .envs
            .iter()
            .filter_map(|x| x.resovle_term(name))
            .next())
    }
}

pub fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}
