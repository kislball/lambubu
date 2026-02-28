use lambubu::{Term, env::TermEnvironment};
use lambubu_macro::term;

pub struct ChurchEnvironment;

impl TermEnvironment for ChurchEnvironment {
    fn resolve_term(&self, name: &str) -> Option<Term> {
        match name {
            "ADD" => Some(self.add()),
            "SUCC" => Some(self.succ()),
            "IF" | "BRANCH" => Some(self.branch()),
            "ZERO" | "0" => Some(self.zero()),
            "FALSE" | "F" => Some(self.bool_false()),
            "TRUE" | "T" => Some(self.bool_true()),
            rest => rest.parse::<u32>().map(|x| self.numeral(x)).ok(),
        }
    }
}

impl ChurchEnvironment {
    pub fn zero(&self) -> Term {
        term!("\\a.\\b.b")
    }

    pub fn bool_false(&self) -> Term {
        self.zero()
    }

    pub fn bool_true(&self) -> Term {
        term!("\\a.\\b.a")
    }

    pub fn numeral(&self, number: u32) -> Term {
        let mut num = Term::app(Term::var("f"), Term::var("x"));

        for _ in 1..number {
            num = Term::app(Term::var("f"), num)
        }

        Term::abs("f", Term::abs("x", num))
    }

    pub fn succ(&self) -> Term {
        term!("\\n.\\f.\\x.(f (n f x))")
    }

    pub fn branch(&self) -> Term {
        term!("\\f.\\a.\\b. (f a b)")
    }

    pub fn add(&self) -> Term {
        term!("\\m.\\n.\\f.\\x.((n f) (m f x))")
    }
}
