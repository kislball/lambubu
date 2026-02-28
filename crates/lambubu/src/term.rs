use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::levels::BruijnLevelsTerm;

const SYMBOL_LAMBDA: char = 'λ';

fn add_prime(s: &str) -> Rc<str> {
    format!("{s}'").into()
}

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
        match self {
            Term::Var(v) => write!(f, "{v}")?,
            Term::Abs(var, term) => write!(f, "{SYMBOL_LAMBDA}{var}.{term}")?,
            Term::Apply(term1, term2) => write!(f, "({term1} {term2})")?,
        };
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
        match self {
            Term::Var(v) => &**v == what,
            Term::Abs(v, body) => body.is_free_variable(what) && &**v != what,
            Term::Apply(t1, t2) => t1.is_free_variable(what) || t2.is_free_variable(what),
        }
    }

    fn rename_free(self, from: &str, to: &str) -> Self {
        if self.is_free_variable(from) {
            match self {
                Term::Var(v) if &*v == from => Term::Var(Rc::from(to)),
                Term::Abs(v, body) if &*v != from => {
                    Term::Abs(v, Rc::new(unwrap_rc(body).rename_free(from, to)))
                }
                Term::Apply(t1, t2) => Term::Apply(
                    Rc::new(unwrap_rc(t1).rename_free(from, to)),
                    Rc::new(unwrap_rc(t2).rename_free(from, to)),
                ),
                _ => self,
            }
        } else {
            self
        }
    }

    pub fn substitute(self, what: &str, with: Term) -> Term {
        match self {
            Term::Var(name) if &*name == what => with,
            Term::Abs(variable, body) if &*variable != what => {
                let (name, body) = if with.is_free_variable(&variable) {
                    let mut fresh: Rc<str> = variable.clone();
                    while with.is_free_variable(&fresh)
                        || body.is_free_variable(&fresh)
                        || &*fresh == what
                    {
                        fresh = add_prime(&fresh);
                    }
                    let body = unwrap_rc(body).rename_free(&variable, &fresh);
                    (fresh, body)
                } else {
                    (variable, unwrap_rc(body))
                };
                Term::Abs(name, Rc::new(body.substitute(what, with)))
            }
            Term::Apply(term1, term2) => Term::Apply(
                Rc::new(unwrap_rc(term1).substitute(what, with.clone())),
                Rc::new(unwrap_rc(term2).substitute(what, with)),
            ),
            _ => self,
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Var(_) | Self::Abs(_, _))
    }

    pub fn is_normal_form(&self) -> bool {
        match self {
            Term::Var(_) => true,
            Term::Abs(_, body) => body.is_normal_form(),
            Term::Apply(t1, t2) => match t1.as_ref() {
                Term::Abs(_, _) => false,
                _ => t1.is_normal_form() && t2.is_normal_form(),
            },
        }
    }

    pub fn reduce_step_call_by_name(self) -> Self {
        match self {
            Self::Apply(t1, t2) => match unwrap_rc(t1) {
                Self::Abs(var, body) => unwrap_rc(body).substitute(&var, unwrap_rc(t2)),
                other => Self::Apply(Rc::new(other.reduce_step_call_by_name()), t2),
            },
            _ => self,
        }
    }

    pub fn reduce_step_normal_order(self) -> Self {
        match self {
            Self::Apply(t1, t2) => match unwrap_rc(t1) {
                Self::Abs(name, body) => unwrap_rc(body).substitute(&name, unwrap_rc(t2)),
                other if !other.is_normal_form() => {
                    Self::Apply(Rc::new(other.reduce_step_normal_order()), t2)
                }
                other => Self::Apply(
                    Rc::new(other),
                    Rc::new(unwrap_rc(t2).reduce_step_normal_order()),
                ),
            },
            Self::Abs(name, body) => {
                Self::Abs(name, Rc::new(unwrap_rc(body).reduce_step_normal_order()))
            }
            other => other,
        }
    }

    pub fn reduce_step_call_by_value(self) -> Self {
        match self {
            Self::Var(_) | Self::Abs(_, _) => self,
            Self::Apply(t1, t2) => {
                let t1_inner = unwrap_rc(t1);
                if let Self::Abs(t1_name, t1_body) = t1_inner {
                    if t2.is_value() {
                        unwrap_rc(t1_body).substitute(&t1_name, unwrap_rc(t2))
                    } else {
                        Self::Apply(
                            Rc::new(Self::Abs(t1_name, t1_body)),
                            Rc::new(unwrap_rc(t2).reduce_step_call_by_value()),
                        )
                    }
                } else {
                    Self::Apply(Rc::new(t1_inner.reduce_step_call_by_value()), t2)
                }
            }
        }
    }

    pub fn reduce_step_applicative_order(self) -> Self {
        match self {
            Self::Apply(t1, t2) => {
                if !t1.is_normal_form() {
                    Self::Apply(Rc::new(unwrap_rc(t1).reduce_step_applicative_order()), t2)
                } else if !t2.is_normal_form() {
                    Self::Apply(t1, Rc::new(unwrap_rc(t2).reduce_step_applicative_order()))
                } else {
                    match unwrap_rc(t1) {
                        Self::Abs(name, body) => unwrap_rc(body).substitute(&name, unwrap_rc(t2)),
                        other => Self::Apply(Rc::new(other), t2),
                    }
                }
            }
            Self::Abs(name, body) => Self::Abs(
                name,
                Rc::new(unwrap_rc(body).reduce_step_applicative_order()),
            ),
            other => other,
        }
    }
}
