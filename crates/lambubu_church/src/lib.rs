//! Implementation of Church numerals and basic algebra
//! The following macros are added:
//! 1. `ADD` --- addition of two Church numerals
//! 2. `SUCC` --- the next Church numeral
//! 3. `IF` and `BRANCH` --- standard branching
//! 4. `ZERO`, `0`, `FALSE`, `F`
//! 5. `TRUE`, `T`
//! 6. Any sequence of digits will be interpreted as a Church numeral
//! 7. `ISZERO` --- checks if a Church numeral is zero
//! 8. `EQ` --- equality check
//! 9. `MULT` --- multiplication
//! 10. `PRED` --- predecessor (n-1)
//! 11. `SUB` --- subtraction
//! 12. `FACT` --- factorial using Y combinator
use lambubu::{env::TermEnvironment, Term};
use lambubu_macro::term;

/// Environment which supplies Church numerals and algebra
pub struct ChurchEnvironment;

impl TermEnvironment for ChurchEnvironment {
    fn resolve_term(&self, name: &str) -> Option<Term> {
        match name {
            "ADD" => Some(self.add()),
            "AND" => Some(self.and()),
            "EQ" => Some(self.eq()),
            "FACT" => Some(self.factorial()),
            "FALSE" | "F" => Some(self.bool_false()),
            "IF" | "BRANCH" => Some(self.branch()),
            "ISZERO" => Some(self.iszero()),
            "MULT" => Some(self.mult()),
            "PRED" => Some(self.pred()),
            "SUB" => Some(self.sub()),
            "SUCC" => Some(self.succ()),
            "TRUE" | "T" => Some(self.bool_true()),
            "Y" => Some(self.y()),
            "Z" => Some(self.z()),
            "ZERO" | "0" => Some(self.zero()),
            rest => rest.parse::<u32>().map(|x| self.numeral(x)).ok(),
        }
    }
}

impl ChurchEnvironment {
    pub fn zero(&self) -> Term {
        term!("\\a.\\b.b")
    }

    pub fn y(&self) -> Term {
        Term::abs(
            "f",
            Term::app(
                Term::abs(
                    "x",
                    Term::app(Term::var("f"), Term::app(Term::var("x"), Term::var("x"))),
                ),
                Term::abs(
                    "x",
                    Term::app(Term::var("f"), Term::app(Term::var("x"), Term::var("x"))),
                ),
            ),
        )
    }

    pub fn z(&self) -> Term {
        Term::abs(
            "f",
            Term::app(
                Term::abs(
                    "x",
                    Term::app(
                        Term::var("f"),
                        Term::abs(
                            "y",
                            Term::app(Term::app(Term::var("x"), Term::var("x")), Term::var("y")),
                        ),
                    ),
                ),
                Term::abs(
                    "x",
                    Term::app(
                        Term::var("f"),
                        Term::abs(
                            "y",
                            Term::app(Term::app(Term::var("x"), Term::var("x")), Term::var("y")),
                        ),
                    ),
                ),
            ),
        )
    }

    pub fn bool_false(&self) -> Term {
        self.zero()
    }

    pub fn bool_true(&self) -> Term {
        term!("\\a.\\b.a")
    }

    pub fn numeral(&self, number: u32) -> Term {
        let mut num = Term::var("x");

        for _ in 0..number {
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

    pub fn and(&self) -> Term {
        Term::abs(
            "a",
            Term::abs(
                "b",
                Term::app(Term::app(Term::var("a"), Term::var("b")), self.zero()),
            ),
        )
    }

    pub fn add(&self) -> Term {
        term!("\\m.\\n.\\f.\\x.((n f) (m f x))")
    }

    pub fn mult(&self) -> Term {
        Term::abs(
            "m",
            Term::abs(
                "n",
                Term::abs(
                    "f",
                    Term::app(Term::var("m"), Term::app(Term::var("n"), Term::var("f"))),
                ),
            ),
        )
    }

    pub fn pred(&self) -> Term {
        Term::abs(
            "n",
            Term::abs(
                "f",
                Term::abs(
                    "x",
                    Term::app(
                        Term::app(
                            Term::app(
                                Term::var("n"),
                                Term::abs(
                                    "g",
                                    Term::abs(
                                        "h",
                                        Term::app(
                                            Term::var("h"),
                                            Term::app(Term::var("g"), Term::var("f")),
                                        ),
                                    ),
                                ),
                            ),
                            Term::abs("u", Term::var("x")),
                        ),
                        Term::abs("u", Term::var("u")),
                    ),
                ),
            ),
        )
    }

    pub fn sub(&self) -> Term {
        Term::abs(
            "m",
            Term::abs(
                "n",
                Term::app(Term::app(Term::var("n"), self.pred()), Term::var("m")),
            ),
        )
    }

    pub fn iszero(&self) -> Term {
        Term::abs(
            "n",
            Term::app(
                Term::app(Term::var("n"), Term::abs("x", self.bool_false())),
                self.bool_true(),
            ),
        )
    }

    pub fn eq(&self) -> Term {
        Term::abs(
            "m",
            Term::abs("n", {
                let iszero_sub_mn = Term::app(
                    self.iszero(),
                    Term::app(Term::app(self.sub(), Term::var("m")), Term::var("n")),
                );
                let iszero_sub_nm = Term::app(
                    self.iszero(),
                    Term::app(Term::app(self.sub(), Term::var("n")), Term::var("m")),
                );
                Term::app(Term::app(self.and(), iszero_sub_mn), iszero_sub_nm)
            }),
        )
    }

    pub fn factorial(&self) -> Term {
        Term::app(
            self.z(),
            Term::abs(
                "f",
                Term::abs(
                    "n",
                    Term::app(
                        Term::app(
                            Term::app(self.branch(), Term::app(self.iszero(), Term::var("n"))),
                            self.numeral(1),
                        ),
                        Term::app(
                            Term::app(self.mult(), Term::var("n")),
                            Term::app(Term::var("f"), Term::app(self.pred(), Term::var("n"))),
                        ),
                    ),
                ),
            ),
        )
    }
}
