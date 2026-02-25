use std::fmt::{self, Display, Formatter};

const SYMBOL_LAMBDA: char = 'λ';

fn add_prime(s: &str) -> String {
    format!("{s}'")
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Term {
    Var(String),
    Abs(String, Box<Term>),
    Apply(Box<Term>, Box<Term>),
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Term::Var(v) => write!(f, "{v}")?,
            Term::Abs(var, term) => write!(f, "({SYMBOL_LAMBDA}{var}.{term})")?,
            Term::Apply(term1, term2) => write!(f, "({term1} {term2})")?,
        };
        Ok(())
    }
}

impl Term {
    pub fn var(s: &str) -> Term {
        Term::Var(s.to_owned())
    }

    pub fn abs(v: &str, body: Term) -> Term {
        Term::Abs(v.to_owned(), Box::new(body))
    }

    pub fn app(t1: Term, t2: Term) -> Term {
        Term::Apply(Box::new(t1), Box::new(t2))
    }

    fn is_free_variable(&self, what: &str) -> bool {
        match self {
            Term::Var(v) => v == what,
            Term::Abs(v, body) => body.is_free_variable(what) && v != what,
            Term::Apply(t1, t2) => t1.is_free_variable(what) || t2.is_free_variable(what),
        }
    }

    fn rename_free(self, from: &str, to: &str) -> Self {
        if self.is_free_variable(from) {
            match self {
                Term::Var(v) if v == from => Term::Var(to.to_owned()),
                Term::Abs(v, body) if v != from => {
                    Term::Abs(v, Box::new(body.rename_free(from, to)))
                }
                Term::Apply(t1, t2) => Term::Apply(
                    Box::new(t1.rename_free(from, to)),
                    Box::new(t2.rename_free(from, to)),
                ),
                _ => self,
            }
        } else {
            self
        }
    }

    pub fn substitute(self, what: &str, with: Term) -> Term {
        match self {
            Term::Var(name) if what == name => with,
            Term::Abs(variable, body) if variable != what => {
                let (name, body) = if with.is_free_variable(&variable) {
                    let mut fresh = variable.clone();
                    while with.is_free_variable(&fresh)
                        || body.is_free_variable(&fresh)
                        || fresh == *what
                    {
                        fresh = add_prime(&fresh);
                    }
                    let body = body.rename_free(&variable, &fresh);

                    (fresh, body)
                } else {
                    (variable, *body)
                };
                Term::Abs(name, Box::new(body.substitute(what, with)))
            }
            Term::Apply(term1, term2) => Term::Apply(
                Box::new(term1.substitute(what, with.clone())),
                Box::new(term2.substitute(what, with)),
            ),
            _ => self,
        }
    }

    pub fn reduce_step_call_by_name(self) -> Self {
        match self {
            Self::Var(_) | Self::Abs(_, _) => self,
            Self::Apply(t1, t2) => {
                let t1 = t1.reduce_step_call_by_name();
                match t1 {
                    Term::Abs(var, body) => body.substitute(&var, *t2),
                    _ => Term::Apply(Box::new(t1), t2),
                }
            }
        }
    }

    pub fn reduce_step_normal_order(self) -> Self {
        match self {
            Self::Var(_) => self,
            Self::Abs(var, body) => Self::Abs(var, Box::new(body.reduce_step_normal_order())),
            Self::Apply(t1, t2) => {
                let e1 = t1.reduce_step_call_by_name();
                match e1 {
                    Self::Abs(var, body) => body.substitute(&var, *t2),
                    _ => Self::Apply(
                        Box::new(e1.reduce_step_normal_order()),
                        Box::new(t2.reduce_step_normal_order()),
                    ),
                }
            }
        }
    }

    pub fn reduce_step_call_by_value(self) -> Self {
        match self {
            Self::Var(_) | Self::Abs(_, _) => self,
            Self::Apply(t1, t2) => {
                let t1 = t1.reduce_step_call_by_value();
                let t2 = t2.reduce_step_call_by_value();
                match t1 {
                    Term::Abs(var, body) => body.substitute(&var, t2),
                    _ => Term::Apply(Box::new(t1), Box::new(t2)),
                }
            }
        }
    }

    pub fn reduce_step_applicative_order(self) -> Self {
        match self {
            Self::Var(_) => self,
            Self::Abs(name, body) => {
                Self::Abs(name, Box::new(body.reduce_step_applicative_order()))
            }
            Self::Apply(t1, t2) => {
                let t1 = t1.reduce_step_applicative_order();
                let t2 = t2.reduce_step_applicative_order();
                match t1 {
                    Term::Abs(var, body) => {
                        body.reduce_step_applicative_order().substitute(&var, t2)
                    }
                    _ => Term::Apply(Box::new(t1), Box::new(t2)),
                }
            }
        }
    }
}
