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
        enum WorkItem<'a> {
            Visit(&'a BruijnLevelsTerm),
            BuildAbs(Rc<str>),
            BuildApply,
            RestoreEnv(usize, Option<Rc<str>>),
            SetupAbs(usize, Rc<str>, &'a BruijnLevelsTerm),
        }

        let mut work_stack: Vec<WorkItem<'_>> = vec![WorkItem::Visit(self)];
        let mut result_stack: Vec<Term> = Vec::new();

        while let Some(item) = work_stack.pop() {
            match item {
                WorkItem::Visit(term) => match term {
                    BruijnLevelsTerm::Var(lvl, name) => {
                        let resolved_name = env.get(lvl).cloned().unwrap_or_else(|| name.clone());
                        result_stack.push(Term::Var(resolved_name));
                    }
                    BruijnLevelsTerm::Abs(lvl, body, name) => {
                        work_stack.push(WorkItem::SetupAbs(*lvl, name.clone(), body));
                    }
                    BruijnLevelsTerm::Apply(t1, t2) => {
                        work_stack.push(WorkItem::BuildApply);
                        work_stack.push(WorkItem::Visit(t2));
                        work_stack.push(WorkItem::Visit(t1));
                    }
                },
                WorkItem::SetupAbs(lvl, name, body) => {
                    let mut distinct_name = name;
                    while env.values().any(|n| n == &distinct_name) {
                        let s: &str = &distinct_name;
                        distinct_name = format!("{}'", s).into();
                    }

                    let old_val = env.insert(lvl, distinct_name.clone());
                    work_stack.push(WorkItem::BuildAbs(distinct_name));
                    work_stack.push(WorkItem::RestoreEnv(lvl, old_val));
                    work_stack.push(WorkItem::Visit(body));
                }
                WorkItem::RestoreEnv(lvl, old_val) => {
                    if let Some(v) = old_val {
                        env.insert(lvl, v);
                    } else {
                        env.remove(&lvl);
                    }
                }
                WorkItem::BuildAbs(name) => {
                    let body = result_stack.pop().unwrap();
                    result_stack.push(Term::Abs(name, Rc::new(body)));
                }
                WorkItem::BuildApply => {
                    let t2 = result_stack.pop().unwrap();
                    let t1 = result_stack.pop().unwrap();
                    result_stack.push(Term::Apply(Rc::new(t1), Rc::new(t2)));
                }
            }
        }

        result_stack.pop().unwrap()
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
        enum WorkItem {
            Visit(Rc<Term>),
            PrepareAbs(Rc<str>, Rc<Term>),
            RestoreDictAndBuildAbs(String, Option<usize>, usize, Rc<str>),
            BuildApply,
        }

        let mut work_stack: Vec<WorkItem> = vec![WorkItem::Visit(term)];
        let mut result_stack: Vec<Rc<BruijnLevelsTerm>> = Vec::new();

        while let Some(item) = work_stack.pop() {
            match item {
                WorkItem::Visit(term) => match term.as_ref() {
                    Term::Var(name) => {
                        result_stack.push(Rc::new(BruijnLevelsTerm::Var(
                            *dictionary.get(name.as_ref()).unwrap(),
                            name.clone(),
                        )));
                    }
                    Term::Abs(name, body) => {
                        work_stack.push(WorkItem::PrepareAbs(name.clone(), body.clone()));
                    }
                    Term::Apply(t1, t2) => {
                        work_stack.push(WorkItem::BuildApply);
                        work_stack.push(WorkItem::Visit(t2.clone()));
                        work_stack.push(WorkItem::Visit(t1.clone()));
                    }
                },
                WorkItem::PrepareAbs(name, body) => {
                    let current_level = *counter;
                    *counter += 1;
                    let old_val = dictionary.insert(name.to_string(), current_level);

                    work_stack.push(WorkItem::RestoreDictAndBuildAbs(
                        name.to_string(),
                        old_val,
                        current_level,
                        name.clone(),
                    ));
                    work_stack.push(WorkItem::Visit(body));
                }
                WorkItem::RestoreDictAndBuildAbs(name_str, old_val, level, name_rc) => {
                    if let Some(v) = old_val {
                        dictionary.insert(name_str, v);
                    } else {
                        dictionary.remove(&name_str);
                    }
                    let body = result_stack.pop().unwrap();
                    result_stack.push(Rc::new(BruijnLevelsTerm::Abs(level, body, name_rc)));
                }
                WorkItem::BuildApply => {
                    let t2 = result_stack.pop().unwrap();
                    let t1 = result_stack.pop().unwrap();
                    result_stack.push(Rc::new(BruijnLevelsTerm::Apply(t1, t2)));
                }
            }
        }

        result_stack.pop().unwrap()
    }

    pub fn substitute(self, what: usize, with: Rc<BruijnLevelsTerm>) -> BruijnLevelsTerm {
        enum WorkItem {
            Process(BruijnLevelsTerm),
            BuildAbs(usize, Rc<str>),
            BuildApply,
        }

        let mut work_stack: Vec<WorkItem> = vec![WorkItem::Process(self)];
        let mut result_stack: Vec<BruijnLevelsTerm> = Vec::new();

        while let Some(item) = work_stack.pop() {
            match item {
                WorkItem::Process(term) => match term {
                    BruijnLevelsTerm::Var(val, _) if val == what => {
                        result_stack.push(unwrap_rc(with.clone()));
                    }
                    BruijnLevelsTerm::Abs(lvl, body, name) if lvl != what => {
                        work_stack.push(WorkItem::BuildAbs(lvl, name));
                        work_stack.push(WorkItem::Process(unwrap_rc(body)));
                    }
                    BruijnLevelsTerm::Apply(a, b) => {
                        work_stack.push(WorkItem::BuildApply);
                        work_stack.push(WorkItem::Process(unwrap_rc(b)));
                        work_stack.push(WorkItem::Process(unwrap_rc(a)));
                    }
                    other => {
                        result_stack.push(other);
                    }
                },
                WorkItem::BuildAbs(lvl, name) => {
                    let body = result_stack.pop().unwrap();
                    result_stack.push(BruijnLevelsTerm::Abs(lvl, Rc::new(body), name));
                }
                WorkItem::BuildApply => {
                    let b = result_stack.pop().unwrap();
                    let a = result_stack.pop().unwrap();
                    result_stack.push(BruijnLevelsTerm::Apply(Rc::new(a), Rc::new(b)));
                }
            }
        }

        result_stack.pop().unwrap()
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Var(_, _) | Self::Abs(_, _, _))
    }

    pub fn is_normal_form(&self) -> bool {
        let mut stack: Vec<&BruijnLevelsTerm> = vec![self];
        while let Some(term) = stack.pop() {
            match term {
                Self::Var(_, _) => {}
                Self::Abs(_, body, _) => stack.push(body),
                Self::Apply(t1, t2) => match t1.as_ref() {
                    Self::Abs(_, _, _) => return false,
                    _ => {
                        stack.push(t1);
                        stack.push(t2);
                    }
                },
            }
        }
        true
    }

    pub fn reduce_step_call_by_name(self) -> (BruijnLevelsTerm, bool) {
        enum Frame {
            ApplyLeft(Rc<BruijnLevelsTerm>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Apply(t1, t2) => match unwrap_rc(t1) {
                    Self::Abs(lvl, body, _) => {
                        let mut result = unwrap_rc(body).substitute(lvl, t2);
                        for frame in frames.into_iter().rev() {
                            let Frame::ApplyLeft(t2) = frame;
                            result = Self::Apply(Rc::new(result), t2);
                        }
                        return (result, true);
                    }
                    other => {
                        frames.push(Frame::ApplyLeft(t2));
                        current = other;
                    }
                },
                _ => {
                    let mut result = current;
                    for frame in frames.into_iter().rev() {
                        let Frame::ApplyLeft(t2) = frame;
                        result = Self::Apply(Rc::new(result), t2);
                    }
                    return (result, false);
                }
            }
        }
    }

    pub fn reduce_step_normal_order(self) -> (BruijnLevelsTerm, bool) {
        enum Frame {
            ApplyLeft(Rc<BruijnLevelsTerm>),
            ApplyRight(Rc<BruijnLevelsTerm>),
            Abs(usize, Rc<str>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Apply(t1, t2) => match unwrap_rc(t1) {
                    Self::Abs(lvl, body, _) => {
                        let mut result = unwrap_rc(body).substitute(lvl, t2);
                        for frame in frames.into_iter().rev() {
                            result = match frame {
                                Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                                Frame::Abs(lvl, name) => Self::Abs(lvl, Rc::new(result), name),
                            };
                        }
                        return (result, true);
                    }
                    other if !other.is_normal_form() => {
                        frames.push(Frame::ApplyLeft(t2));
                        current = other;
                    }
                    other => {
                        frames.push(Frame::ApplyRight(Rc::new(other)));
                        current = unwrap_rc(t2);
                    }
                },
                Self::Abs(lvl, body, name) => {
                    frames.push(Frame::Abs(lvl, name));
                    current = unwrap_rc(body);
                }
                other => {
                    let mut result = other;
                    for frame in frames.into_iter().rev() {
                        result = match frame {
                            Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                            Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                            Frame::Abs(lvl, name) => Self::Abs(lvl, Rc::new(result), name),
                        };
                    }
                    return (result, false);
                }
            }
        }
    }

    pub fn reduce_step_call_by_value(self) -> (BruijnLevelsTerm, bool) {
        enum Frame {
            ApplyLeft(Rc<BruijnLevelsTerm>),
            ApplyRightAbs(usize, Rc<BruijnLevelsTerm>, Rc<str>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Var(_, _) | Self::Abs(_, _, _) => {
                    let mut result = current;
                    for frame in frames.into_iter().rev() {
                        result = match frame {
                            Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                            Frame::ApplyRightAbs(lvl, body, name) => {
                                Self::Apply(Rc::new(Self::Abs(lvl, body, name)), Rc::new(result))
                            }
                        };
                    }
                    return (result, false);
                }
                Self::Apply(t1, t2) => {
                    let t1_inner = unwrap_rc(t1);
                    if let Self::Abs(lvl, body, name) = t1_inner {
                        if t2.is_value() {
                            let mut result = unwrap_rc(body).substitute(lvl, t2);
                            for frame in frames.into_iter().rev() {
                                result = match frame {
                                    Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                    Frame::ApplyRightAbs(lvl, body, name) => Self::Apply(
                                        Rc::new(Self::Abs(lvl, body, name)),
                                        Rc::new(result),
                                    ),
                                };
                            }
                            return (result, true);
                        } else {
                            frames.push(Frame::ApplyRightAbs(lvl, body, name));
                            current = unwrap_rc(t2);
                        }
                    } else {
                        frames.push(Frame::ApplyLeft(t2));
                        current = t1_inner;
                    }
                }
            }
        }
    }

    pub fn reduce_step_applicative_order(self) -> (BruijnLevelsTerm, bool) {
        enum Frame {
            ApplyLeft(Rc<BruijnLevelsTerm>),
            ApplyRight(Rc<BruijnLevelsTerm>),
            Abs(usize, Rc<str>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Apply(t1, t2) => {
                    if !t1.is_normal_form() {
                        frames.push(Frame::ApplyLeft(t2));
                        current = unwrap_rc(t1);
                    } else if !t2.is_normal_form() {
                        frames.push(Frame::ApplyRight(t1));
                        current = unwrap_rc(t2);
                    } else {
                        match unwrap_rc(t1) {
                            Self::Abs(lvl, body, _) => {
                                let mut result = unwrap_rc(body).substitute(lvl, t2);
                                for frame in frames.into_iter().rev() {
                                    result = match frame {
                                        Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                        Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                                        Frame::Abs(lvl, name) => {
                                            Self::Abs(lvl, Rc::new(result), name)
                                        }
                                    };
                                }
                                return (result, true);
                            }
                            other => {
                                let mut result = Self::Apply(Rc::new(other), t2);
                                for frame in frames.into_iter().rev() {
                                    result = match frame {
                                        Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                        Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                                        Frame::Abs(lvl, name) => {
                                            Self::Abs(lvl, Rc::new(result), name)
                                        }
                                    };
                                }
                                return (result, false);
                            }
                        }
                    }
                }
                Self::Abs(lvl, body, name) => {
                    frames.push(Frame::Abs(lvl, name));
                    current = unwrap_rc(body);
                }
                other => {
                    let mut result = other;
                    for frame in frames.into_iter().rev() {
                        result = match frame {
                            Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                            Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                            Frame::Abs(lvl, name) => Self::Abs(lvl, Rc::new(result), name),
                        };
                    }
                    return (result, false);
                }
            }
        }
    }
}

fn unwrap_rc(rc: Rc<BruijnLevelsTerm>) -> BruijnLevelsTerm {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}
