use crate::Term;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone, Debug, Eq)]
pub enum BruijnLevelsTerm {
    Var(u16, Rc<str>),
    Abs(u16, Rc<BruijnLevelsTerm>, Rc<str>),
    Apply(Rc<BruijnLevelsTerm>, Rc<BruijnLevelsTerm>),
}

impl PartialEq for BruijnLevelsTerm {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Var(l1, _), Self::Var(l2, _)) => l1 == l2,
            (Self::Abs(l1, b1, _), Self::Abs(l2, b2, _)) => l1 == l2 && b1 == b2,
            (Self::Apply(a1, b1), Self::Apply(a2, b2)) => a1 == a2 && b1 == b2,
            _ => false,
        }
    }
}

impl std::hash::Hash for BruijnLevelsTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Var(l, _) => {
                0u8.hash(state);
                l.hash(state);
            }
            Self::Abs(l, body, _) => {
                1u8.hash(state);
                l.hash(state);
                body.hash(state);
            }
            Self::Apply(a, b) => {
                2u8.hash(state);
                a.hash(state);
                b.hash(state);
            }
        }
    }
}

impl From<BruijnLevelsTerm> for Term {
    fn from(value: BruijnLevelsTerm) -> Self {
        match value {
            BruijnLevelsTerm::Var(_, name) => Term::Var(name),
            BruijnLevelsTerm::Abs(_, body, name) => {
                Term::Abs(name, Rc::new((*body).clone().into()))
            }
            BruijnLevelsTerm::Apply(t1, t2) => {
                Term::Apply(Rc::new((*t1).clone().into()), Rc::new((*t2).clone().into()))
            }
        }
    }
}

impl From<Term> for BruijnLevelsTerm {
    fn from(value: Term) -> Self {
        let mut counter = 0;
        (*BruijnLevelsTerm::from_term(Rc::new(value), &HashMap::new(), &mut counter)).clone()
    }
}

impl From<Rc<Term>> for BruijnLevelsTerm {
    fn from(value: Rc<Term>) -> Self {
        let mut counter = 0;
        (*BruijnLevelsTerm::from_term(value, &HashMap::new(), &mut counter)).clone()
    }
}

impl BruijnLevelsTerm {
    pub fn from_open_term(term: Term) -> Rc<BruijnLevelsTerm> {
        let mut free_vars: Vec<String> = Vec::new();
        Self::collect_free_vars(&term, &HashSet::new(), &mut free_vars);
        let mut dict = HashMap::new();
        for (i, v) in free_vars.iter().enumerate() {
            dict.insert(v.clone(), i as u16);
        }
        let mut counter = free_vars.len() as u16;
        Self::from_term(Rc::new(term), &dict, &mut counter)
    }

    fn collect_free_vars(term: &Term, bound: &HashSet<String>, free: &mut Vec<String>) {
        match term {
            Term::Var(v) => {
                let s = v.to_string();
                if !bound.contains(&s) && !free.contains(&s) {
                    free.push(s);
                }
            }
            Term::Abs(v, body) => {
                let mut new_bound = bound.clone();
                new_bound.insert(v.to_string());
                Self::collect_free_vars(body, &new_bound, free);
            }
            Term::Apply(t1, t2) => {
                Self::collect_free_vars(t1, bound, free);
                Self::collect_free_vars(t2, bound, free);
            }
        }
    }

    fn from_term(
        term: Rc<Term>,
        dictionary: &HashMap<String, u16>,
        counter: &mut u16,
    ) -> Rc<BruijnLevelsTerm> {
        match term.as_ref() {
            Term::Var(name) => Rc::new(BruijnLevelsTerm::Var(
                *dictionary.get(name.as_ref()).unwrap(),
                name.clone(),
            )),
            Term::Abs(name, body) => {
                let current_level = *counter;
                *counter += 1;
                let mut new_hash = dictionary.clone();
                new_hash.insert(name.to_string(), current_level);
                Rc::new(BruijnLevelsTerm::Abs(
                    current_level,
                    Self::from_term(body.clone(), &new_hash, counter),
                    name.clone(),
                ))
            }
            Term::Apply(t1, t2) => {
                let t1 = BruijnLevelsTerm::from_term(t1.clone(), dictionary, counter);
                let t2 = BruijnLevelsTerm::from_term(t2.clone(), dictionary, counter);
                Rc::new(BruijnLevelsTerm::Apply(t1, t2))
            }
        }
    }

    pub fn substitute(
        self: Rc<BruijnLevelsTerm>,
        what: u16,
        with: Rc<BruijnLevelsTerm>,
    ) -> Rc<BruijnLevelsTerm> {
        match self.as_ref() {
            BruijnLevelsTerm::Var(val, _) if *val == what => with,
            BruijnLevelsTerm::Abs(lvl, body, name) => {
                let new_body = Self::substitute(body.clone(), what, with);
                if Rc::ptr_eq(&new_body, body) {
                    self
                } else {
                    Rc::new(BruijnLevelsTerm::Abs(*lvl, new_body, name.clone()))
                }
            }
            BruijnLevelsTerm::Apply(a, b) => {
                let new_a = Self::substitute(a.clone(), what, with.clone());
                let new_b = Self::substitute(b.clone(), what, with);
                if Rc::ptr_eq(&new_a, a) && Rc::ptr_eq(&new_b, b) {
                    self
                } else {
                    Rc::new(BruijnLevelsTerm::Apply(new_a, new_b))
                }
            }
            _ => self,
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Var(_, _) | Self::Abs(_, _, _))
    }

    pub fn is_normal_form(self: Rc<Self>) -> bool {
        match self.as_ref() {
            Self::Var(_, _) => true,
            Self::Abs(_, body, _) => body.clone().is_normal_form(),
            Self::Apply(t1, t2) => match t1.as_ref() {
                Self::Abs(_, _, _) => false,
                _ => t1.clone().is_normal_form() && t2.clone().is_normal_form(),
            },
        }
    }

    pub fn reduce_step_call_by_name(self: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        match self.as_ref() {
            Self::Apply(t1, t2) => match t1.as_ref() {
                Self::Abs(lvl, body, _) => body.clone().substitute(*lvl, t2.clone()),
                _ => Rc::new(Self::Apply(
                    t1.clone().reduce_step_call_by_name(),
                    t2.clone(),
                )),
            },
            _ => self,
        }
    }

    pub fn reduce_step_normal_order(self: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        match self.as_ref() {
            Self::Apply(t1, t2) => match t1.as_ref() {
                Self::Abs(lvl, body, _) => body.clone().substitute(*lvl, t2.clone()),
                _ if !t1.clone().is_normal_form() => Rc::new(Self::Apply(
                    t1.clone().reduce_step_normal_order(),
                    t2.clone(),
                )),
                _ => Rc::new(Self::Apply(
                    t1.clone(),
                    t2.clone().reduce_step_normal_order(),
                )),
            },
            Self::Abs(lvl, body, name) => Rc::new(Self::Abs(
                *lvl,
                body.clone().reduce_step_normal_order(),
                name.clone(),
            )),
            _ => self,
        }
    }

    pub fn reduce_step_call_by_value(self: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        match self.as_ref() {
            Self::Var(_, _) | Self::Abs(_, _, _) => self,
            Self::Apply(t1, t2) => {
                if let Self::Abs(lvl, body, _) = t1.as_ref() {
                    if t2.is_value() {
                        body.clone().substitute(*lvl, t2.clone())
                    } else {
                        Rc::new(Self::Apply(
                            t1.clone(),
                            t2.clone().reduce_step_call_by_value(),
                        ))
                    }
                } else {
                    Rc::new(Self::Apply(
                        t1.clone().reduce_step_call_by_value(),
                        t2.clone(),
                    ))
                }
            }
        }
    }

    pub fn reduce_step_applicative_order(self: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        match self.as_ref() {
            Self::Apply(t1, t2) => {
                if !t1.clone().is_normal_form() {
                    Rc::new(Self::Apply(
                        t1.clone().reduce_step_applicative_order(),
                        t2.clone(),
                    ))
                } else if !t2.clone().is_normal_form() {
                    Rc::new(Self::Apply(
                        t1.clone(),
                        t2.clone().reduce_step_applicative_order(),
                    ))
                } else {
                    match t1.as_ref() {
                        Self::Abs(lvl, body, _) => body.clone().substitute(*lvl, t2.clone()),
                        _ => self,
                    }
                }
            }
            Self::Abs(lvl, body, name) => Rc::new(Self::Abs(
                *lvl,
                body.clone().reduce_step_applicative_order(),
                name.clone(),
            )),
            _ => self,
        }
    }
}
