//! Term implementation
//! All strategies perform a single step at a time
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::levels::BruijnLevelsTerm;

const SYMBOL_LAMBDA: char = 'λ';

fn unwrap_rc(rc: Rc<Term>) -> Term {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Term {
    Var(Rc<str>),
    Abs(Rc<str>, Rc<Term>),
    Apply(Rc<Term>, Rc<Term>),
}

impl Hash for Term {
    fn hash<H: Hasher>(&self, state: &mut H) {
        BruijnLevelsTerm::from(self.clone()).hash(state)
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        enum Action<'a> {
            Visit(&'a Term),
            WriteStr(&'static str),
            WriteRcStr(&'a Rc<str>),
            WriteChar(char),
        }

        let mut stack: Vec<Action<'_>> = vec![Action::Visit(self)];

        while let Some(action) = stack.pop() {
            match action {
                Action::Visit(term) => match term {
                    Term::Var(v) => {
                        write!(f, "{v}")?;
                    }
                    Term::Abs(var, body) => {
                        stack.push(Action::Visit(body));
                        stack.push(Action::WriteStr("."));
                        stack.push(Action::WriteRcStr(var));
                        stack.push(Action::WriteChar(SYMBOL_LAMBDA));
                    }
                    Term::Apply(term1, term2) => {
                        stack.push(Action::WriteStr(")"));
                        stack.push(Action::Visit(term2));
                        stack.push(Action::WriteStr(" "));
                        stack.push(Action::Visit(term1));
                        stack.push(Action::WriteStr("("));
                    }
                },
                Action::WriteStr(s) => write!(f, "{s}")?,
                Action::WriteRcStr(s) => write!(f, "{s}")?,
                Action::WriteChar(c) => write!(f, "{c}")?,
            }
        }
        Ok(())
    }
}

impl Term {
    pub fn var(s: &str) -> Term {
        Term::Var(Rc::from(s))
    }

    pub fn abs(v: &str, body: Term) -> Term {
        Term::Abs(Rc::from(v), Rc::new(body))
    }

    pub fn app(t1: Term, t2: Term) -> Term {
        Term::Apply(Rc::new(t1), Rc::new(t2))
    }

    fn is_free_variable(&self, what: &str) -> bool {
        let mut stack: Vec<&Term> = vec![self];
        while let Some(term) = stack.pop() {
            match term {
                Term::Var(v) => {
                    if &**v == what {
                        return true;
                    }
                }
                Term::Abs(v, body) => {
                    if &**v != what {
                        stack.push(body);
                    }
                }
                Term::Apply(t1, t2) => {
                    stack.push(t1);
                    stack.push(t2);
                }
            }
        }
        false
    }

    fn rename_free(self, from: &str, to: &str) -> Self {
        enum WorkItem {
            Process(Term),
            BuildAbs(Rc<str>),
            BuildApply,
        }

        let mut work_stack: Vec<WorkItem> = vec![WorkItem::Process(self)];
        let mut result_stack: Vec<Term> = Vec::new();

        while let Some(item) = work_stack.pop() {
            match item {
                WorkItem::Process(term) => {
                    if !term.is_free_variable(from) {
                        result_stack.push(term);
                    } else {
                        match term {
                            Term::Var(v) if &*v == from => {
                                result_stack.push(Term::Var(Rc::from(to)));
                            }
                            Term::Abs(v, body) if &*v != from => {
                                work_stack.push(WorkItem::BuildAbs(v));
                                work_stack.push(WorkItem::Process(unwrap_rc(body)));
                            }
                            Term::Apply(t1, t2) => {
                                work_stack.push(WorkItem::BuildApply);
                                work_stack.push(WorkItem::Process(unwrap_rc(t2)));
                                work_stack.push(WorkItem::Process(unwrap_rc(t1)));
                            }
                            _ => {
                                result_stack.push(term);
                            }
                        }
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

    pub fn substitute(self, what: &str, with: Term) -> Term {
        enum WorkItem {
            Process(Term),
            BuildAbs(Rc<str>),
            BuildApply,
        }

        let mut work_stack: Vec<WorkItem> = vec![WorkItem::Process(self)];
        let mut result_stack: Vec<Term> = Vec::new();

        while let Some(item) = work_stack.pop() {
            match item {
                WorkItem::Process(term) => match term {
                    Term::Var(name) => {
                        if &*name == what {
                            result_stack.push(with.clone());
                        } else {
                            result_stack.push(Term::Var(name));
                        }
                    }
                    Term::Abs(variable, body) if &*variable != what => {
                        let (name, body_term) = if with.is_free_variable(&variable) {
                            let mut fresh = String::from(variable.as_ref());
                            while with.is_free_variable(&fresh)
                                || body.is_free_variable(&fresh)
                                || &*fresh == what
                            {
                                fresh.push('\'');
                            }
                            let body = unwrap_rc(body).rename_free(&variable, &fresh);
                            (Into::<Rc<str>>::into(fresh), body)
                        } else {
                            (variable, unwrap_rc(body))
                        };
                        work_stack.push(WorkItem::BuildAbs(name));
                        work_stack.push(WorkItem::Process(body_term));
                    }
                    Term::Abs(variable, body) => {
                        result_stack.push(Term::Abs(variable, body));
                    }
                    Term::Apply(t1, t2) => {
                        work_stack.push(WorkItem::BuildApply);
                        work_stack.push(WorkItem::Process(unwrap_rc(t2)));
                        work_stack.push(WorkItem::Process(unwrap_rc(t1)));
                    }
                },
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

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Var(_) | Self::Abs(_, _))
    }

    pub fn is_normal_form(&self) -> bool {
        let mut stack: Vec<&Term> = vec![self];
        while let Some(term) = stack.pop() {
            match term {
                Term::Var(_) => {}
                Term::Abs(_, body) => stack.push(body),
                Term::Apply(t1, t2) => match t1.as_ref() {
                    Term::Abs(_, _) => return false,
                    _ => {
                        stack.push(t1);
                        stack.push(t2);
                    }
                },
            }
        }
        true
    }

    pub fn reduce_step_call_by_name(self) -> (Self, bool) {
        enum Frame {
            ApplyLeft(Rc<Term>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Apply(t1, t2) => match unwrap_rc(t1) {
                    Self::Abs(var, body) => {
                        let mut result = unwrap_rc(body).substitute(&var, unwrap_rc(t2));
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
                    let changed = false;
                    for frame in frames.into_iter().rev() {
                        let Frame::ApplyLeft(t2) = frame;
                        result = Self::Apply(Rc::new(result), t2);
                    }
                    return (result, changed);
                }
            }
        }
    }

    pub fn reduce_step_normal_order(self) -> (Self, bool) {
        enum Frame {
            ApplyLeft(Rc<Term>),
            ApplyRight(Rc<Term>),
            Abs(Rc<str>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Apply(t1, t2) => match unwrap_rc(t1) {
                    Self::Abs(name, body) => {
                        let mut result = unwrap_rc(body).substitute(&name, unwrap_rc(t2));
                        for frame in frames.into_iter().rev() {
                            result = match frame {
                                Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                                Frame::Abs(name) => Self::Abs(name, Rc::new(result)),
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
                Self::Abs(name, body) => {
                    frames.push(Frame::Abs(name));
                    current = unwrap_rc(body);
                }
                other => {
                    let mut result = other;
                    for frame in frames.into_iter().rev() {
                        result = match frame {
                            Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                            Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                            Frame::Abs(name) => Self::Abs(name, Rc::new(result)),
                        };
                    }
                    return (result, false);
                }
            }
        }
    }

    pub fn reduce_step_call_by_value(self) -> (Self, bool) {
        enum Frame {
            ApplyLeft(Rc<Term>),
            ApplyRightAbs(Rc<str>, Rc<Term>),
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut current = self;

        loop {
            match current {
                Self::Var(_) | Self::Abs(_, _) => {
                    let mut result = current;
                    for frame in frames.into_iter().rev() {
                        result = match frame {
                            Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                            Frame::ApplyRightAbs(name, body) => {
                                Self::Apply(Rc::new(Self::Abs(name, body)), Rc::new(result))
                            }
                        };
                    }
                    return (result, false);
                }
                Self::Apply(t1, t2) => {
                    let t1_inner = unwrap_rc(t1);
                    if let Self::Abs(t1_name, t1_body) = t1_inner {
                        if t2.is_value() {
                            let mut result = unwrap_rc(t1_body).substitute(&t1_name, unwrap_rc(t2));
                            for frame in frames.into_iter().rev() {
                                result = match frame {
                                    Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                    Frame::ApplyRightAbs(name, body) => {
                                        Self::Apply(Rc::new(Self::Abs(name, body)), Rc::new(result))
                                    }
                                };
                            }
                            return (result, true);
                        } else {
                            frames.push(Frame::ApplyRightAbs(t1_name, t1_body));
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

    pub fn reduce_step_applicative_order(self) -> (Self, bool) {
        enum Frame {
            ApplyLeft(Rc<Term>),
            ApplyRight(Rc<Term>),
            Abs(Rc<str>),
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
                            Self::Abs(name, body) => {
                                let mut result = unwrap_rc(body).substitute(&name, unwrap_rc(t2));
                                for frame in frames.into_iter().rev() {
                                    result = match frame {
                                        Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                                        Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                                        Frame::Abs(name) => Self::Abs(name, Rc::new(result)),
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
                                        Frame::Abs(name) => Self::Abs(name, Rc::new(result)),
                                    };
                                }
                                return (result, false);
                            }
                        }
                    }
                }
                Self::Abs(name, body) => {
                    frames.push(Frame::Abs(name));
                    current = unwrap_rc(body);
                }
                other => {
                    let mut result = other;
                    for frame in frames.into_iter().rev() {
                        result = match frame {
                            Frame::ApplyLeft(t2) => Self::Apply(Rc::new(result), t2),
                            Frame::ApplyRight(t1) => Self::Apply(t1, Rc::new(result)),
                            Frame::Abs(name) => Self::Abs(name, Rc::new(result)),
                        };
                    }
                    return (result, false);
                }
            }
        }
    }
}
