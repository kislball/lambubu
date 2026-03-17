//! Bruijn levels implementation
//! All strategies perform a single reduction at a time
use crate::Term;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone, Debug, Eq)]
pub enum BruijnLevelsTerm {
    Var(usize, Rc<str>),
    Abs(usize, Rc<BruijnLevelsTerm>, Rc<str>),
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
        value.to_term(&mut HashMap::new())
    }
}

impl From<Term> for BruijnLevelsTerm {
    fn from(value: Term) -> Self {
        unwrap_rc(BruijnLevelsTerm::from_term(
            Rc::new(value),
            &mut HashMap::new(),
            &mut 0,
        ))
    }
}

impl From<Rc<Term>> for BruijnLevelsTerm {
    fn from(value: Rc<Term>) -> Self {
        unwrap_rc(BruijnLevelsTerm::from_term(
            value,
            &mut HashMap::new(),
            &mut 0,
        ))
    }
}

impl BruijnLevelsTerm {
    fn to_term(&self, env: &mut HashMap<usize, Rc<str>>) -> Term {
        match self {
            BruijnLevelsTerm::Var(lvl, name) => {
                let resolved_name = env.get(lvl).cloned().unwrap_or_else(|| name.clone());
                Term::Var(resolved_name)
            }
            BruijnLevelsTerm::Abs(lvl, body, name) => {
                let mut distinct_name = name.clone();
                while env.values().any(|n| n == &distinct_name) {
                    let s: &str = &distinct_name;
                    distinct_name = format!("{}'", s).into();
                }

                env.insert(*lvl, distinct_name.clone());
                let body_term = body.to_term(env);
                env.remove(lvl);

                Term::Abs(distinct_name, Rc::new(body_term))
            }
            BruijnLevelsTerm::Apply(t1, t2) => {
                Term::Apply(Rc::new(t1.to_term(env)), Rc::new(t2.to_term(env)))
            }
        }
    }

    pub fn from_open_term(term: Term) -> BruijnLevelsTerm {
        let mut free_vars: Vec<String> = Vec::new();
        Self::collect_free_vars(&term, &HashSet::new(), &mut free_vars);
        let mut dict = HashMap::new();
        for (i, v) in free_vars.iter().enumerate() {
            dict.insert(v.clone(), i);
        }
        let mut counter = free_vars.len();
        unwrap_rc(Self::from_term(Rc::new(term), &mut dict, &mut counter))
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
        dictionary: &mut HashMap<String, usize>,
        counter: &mut usize,
    ) -> Rc<BruijnLevelsTerm> {
        match term.as_ref() {
            Term::Var(name) => Rc::new(BruijnLevelsTerm::Var(
                *dictionary.get(name.as_ref()).unwrap(),
                name.clone(),
            )),
            Term::Abs(name, body) => {
                let current_level = *counter;
                *counter += 1;
                let old_val = dictionary.insert(name.to_string(), current_level);
                let body_res = Self::from_term(body.clone(), dictionary, counter);
                if let Some(v) = old_val {
                    dictionary.insert(name.to_string(), v);
                } else {
                    dictionary.remove(name.as_ref());
                }
                Rc::new(BruijnLevelsTerm::Abs(current_level, body_res, name.clone()))
            }
            Term::Apply(t1, t2) => {
                let t1 = BruijnLevelsTerm::from_term(t1.clone(), dictionary, counter);
                let t2 = BruijnLevelsTerm::from_term(t2.clone(), dictionary, counter);
                Rc::new(BruijnLevelsTerm::Apply(t1, t2))
            }
        }
    }

    pub fn substitute(self, what: usize, with: Rc<BruijnLevelsTerm>) -> BruijnLevelsTerm {
        match self {
            BruijnLevelsTerm::Var(val, _) if val == what => unwrap_rc(with),
            BruijnLevelsTerm::Abs(lvl, body, name) if lvl != what => {
                BruijnLevelsTerm::Abs(lvl, Rc::new(unwrap_rc(body).substitute(what, with)), name)
            }
            BruijnLevelsTerm::Apply(a, b) => BruijnLevelsTerm::Apply(
                Rc::new(unwrap_rc(a).substitute(what, with.clone())),
                Rc::new(unwrap_rc(b).substitute(what, with)),
            ),
            _ => self,
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Var(_, _) | Self::Abs(_, _, _))
    }

    pub fn is_normal_form(&self) -> bool {
        match self {
            Self::Var(_, _) => true,
            Self::Abs(_, body, _) => body.is_normal_form(),
            Self::Apply(t1, t2) => match t1.as_ref() {
                Self::Abs(_, _, _) => false,
                _ => t1.is_normal_form() && t2.is_normal_form(),
            },
        }
    }

    pub fn reduce_step_call_by_name(self) -> (BruijnLevelsTerm, bool) {
        match self {
            Self::Apply(t1, t2) => match unwrap_rc(t1) {
                Self::Abs(lvl, body, _) => (unwrap_rc(body).substitute(lvl, t2), true),
                other => {
                    let (reduced, changed) = other.reduce_step_call_by_name();
                    (Self::Apply(Rc::new(reduced), t2), changed)
                }
            },
            _ => (self, false),
        }
    }

    pub fn reduce_step_normal_order(self) -> (BruijnLevelsTerm, bool) {
        match self {
            Self::Apply(t1, t2) => match unwrap_rc(t1) {
                Self::Abs(lvl, body, _) => (unwrap_rc(body).substitute(lvl, t2), true),
                other if !other.is_normal_form() => {
                    let (reduced, changed) = other.reduce_step_normal_order();
                    (Self::Apply(Rc::new(reduced), t2), changed)
                }
                other => {
                    let (reduced, changed) = unwrap_rc(t2).reduce_step_normal_order();
                    (Self::Apply(Rc::new(other), Rc::new(reduced)), changed)
                }
            },
            Self::Abs(lvl, body, name) => {
                let (reduced, changed) = unwrap_rc(body).reduce_step_normal_order();
                (Self::Abs(lvl, Rc::new(reduced), name), changed)
            }
            _ => (self, false),
        }
    }

    pub fn reduce_step_call_by_value(self) -> (BruijnLevelsTerm, bool) {
        match self {
            Self::Var(_, _) | Self::Abs(_, _, _) => (self, false),
            Self::Apply(t1, t2) => {
                let t1_inner = unwrap_rc(t1);
                if let Self::Abs(lvl, body, name) = t1_inner {
                    if t2.is_value() {
                        (unwrap_rc(body).substitute(lvl, t2), true)
                    } else {
                        let (reduced, changed) = unwrap_rc(t2).reduce_step_call_by_value();
                        (
                            Self::Apply(Rc::new(Self::Abs(lvl, body, name)), Rc::new(reduced)),
                            changed,
                        )
                    }
                } else {
                    let (reduced, changed) = t1_inner.reduce_step_call_by_value();
                    (Self::Apply(Rc::new(reduced), t2), changed)
                }
            }
        }
    }

    pub fn reduce_step_applicative_order(self) -> (BruijnLevelsTerm, bool) {
        match self {
            Self::Apply(t1, t2) => {
                if !t1.is_normal_form() {
                    let (reduced, changed) = unwrap_rc(t1).reduce_step_applicative_order();
                    (Self::Apply(Rc::new(reduced), t2), changed)
                } else if !t2.is_normal_form() {
                    let (reduced, changed) = unwrap_rc(t2).reduce_step_applicative_order();
                    (Self::Apply(t1, Rc::new(reduced)), changed)
                } else {
                    match unwrap_rc(t1) {
                        Self::Abs(lvl, body, _) => (unwrap_rc(body).substitute(lvl, t2), true),
                        other => (Self::Apply(Rc::new(other), t2), false),
                    }
                }
            }
            Self::Abs(lvl, body, name) => {
                let (reduced, changed) = unwrap_rc(body).reduce_step_applicative_order();
                (Self::Abs(lvl, Rc::new(reduced), name), changed)
            }
            _ => (self, false),
        }
    }
}

fn unwrap_rc(rc: Rc<BruijnLevelsTerm>) -> BruijnLevelsTerm {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}
