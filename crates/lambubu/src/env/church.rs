use crate::{Term, env::TermEnvironment};

pub struct ChurchEnvironment;

impl TermEnvironment for ChurchEnvironment {
    fn resovle_term(&self, name: &str) -> Option<Term> {
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
        Term::abs("f", Term::abs("x", Term::var("x")))
    }

    pub fn bool_false(&self) -> Term {
        self.zero()
    }

    pub fn bool_true(&self) -> Term {
        Term::abs("a", Term::abs("b", Term::var("a")))
    }

    pub fn numeral(&self, number: u32) -> Term {
        let mut num = Term::app(Term::var("f"), Term::var("x"));

        for _ in 1..number {
            num = Term::app(Term::var("f"), num)
        }

        Term::abs("f", Term::abs("x", num))
    }

    pub fn succ(&self) -> Term {
        Term::abs(
            "n",
            Term::abs(
                "f",
                Term::abs(
                    "x",
                    Term::app(
                        Term::var("f"),
                        Term::app(Term::app(Term::var("n"), Term::var("f")), Term::var("x")),
                    ),
                ),
            ),
        )
    }

    pub fn branch(&self) -> Term {
        Term::abs(
            "f",
            Term::abs(
                "a",
                Term::abs(
                    "b",
                    Term::app(Term::app(Term::var("f"), Term::var("a")), Term::var("b")),
                ),
            ),
        )
    }

    pub fn add(&self) -> Term {
        Term::abs(
            "m",
            Term::abs(
                "n",
                Term::abs(
                    "f",
                    Term::abs(
                        "x",
                        Term::app(
                            Term::app(Term::var("n"), Term::var("f")),
                            Term::app(Term::app(Term::var("m"), Term::var("f")), Term::var("x")),
                        ),
                    ),
                ),
            ),
        )
    }
}
