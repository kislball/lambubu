use lambubu::BruijnLevelsTerm;
use lambubu::Term;

fn reduce_to_normal(mut t: Term) -> Term {
    while !t.is_normal_form() {
        t = t.reduce_step_applicative_order();
    }
    t
}

fn bruijn_step(t: Term) -> Term {
    let b = BruijnLevelsTerm::from_open_term(t);
    Term::from((*b.reduce_step_applicative_order()).clone())
}

fn bruijn_reduce_to_normal(t: Term) -> Term {
    let mut b = BruijnLevelsTerm::from_open_term(t);
    loop {
        if b.is_normal_form() {
            break;
        }
        b = b.reduce_step_applicative_order();
    }
    Term::from((*b).clone())
}

// (λx.x) a →_ao a
#[test]
fn ao_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(term.reduce_step_applicative_order(), Term::var("a"));
}

#[test]
fn bruijn_ao_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(bruijn_step(term), Term::var("a"));
}

// λz.(λx.x) a →_ao λz.a  (reduces under λ, unlike call-by-name/call-by-value)
#[test]
fn ao_reduces_under_lambda() {
    let term = Term::abs(
        "z",
        Term::app(Term::abs("x", Term::var("x")), Term::var("a")),
    );
    assert_eq!(
        term.reduce_step_applicative_order()
            .reduce_step_applicative_order(),
        Term::abs("z", Term::var("a"))
    );
}

#[test]
fn bruijn_ao_reduces_under_lambda() {
    let term = Term::abs(
        "z",
        Term::app(Term::abs("x", Term::var("x")), Term::var("a")),
    );
    assert_eq!(
        bruijn_step(bruijn_step(term)),
        Term::abs("z", Term::var("a"))
    );
}

// (λx.x x) ((λy.y) a) →_ao (a a)
// Argument (λy.y) a is reduced to a before substitution.
// Call-by-name would give ((λy.y) a) ((λy.y) a) instead.
#[test]
fn ao_reduces_argument_before_substitution() {
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        Term::app(Term::abs("y", Term::var("y")), Term::var("a")),
    );
    assert_eq!(
        term.reduce_step_applicative_order()
            .reduce_step_applicative_order(),
        Term::app(Term::var("a"), Term::var("a"))
    );
}

#[test]
fn bruijn_ao_reduces_argument_before_substitution() {
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        Term::app(Term::abs("y", Term::var("y")), Term::var("a")),
    );
    assert_eq!(
        bruijn_step(bruijn_step(term)),
        Term::app(Term::var("a"), Term::var("a"))
    );
}

// (λx.λy.x) a b →*_ao a  (K combinator)
#[test]
fn ao_full_k_combinator() {
    let term = Term::app(
        Term::app(
            Term::abs("x", Term::abs("y", Term::var("x"))),
            Term::var("a"),
        ),
        Term::var("b"),
    );
    assert_eq!(reduce_to_normal(term), Term::var("a"));
}

#[test]
fn bruijn_ao_full_k_combinator() {
    let term = Term::app(
        Term::app(
            Term::abs("x", Term::abs("y", Term::var("x"))),
            Term::var("a"),
        ),
        Term::var("b"),
    );
    assert_eq!(bruijn_reduce_to_normal(term), Term::var("a"));
}

// (λf.λx.f (f x)) (λy.y) →*_ao λx.x  (church numeral 2 applied to identity)
#[test]
fn ao_full_church_2_applied_to_id() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(Term::var("f"), Term::app(Term::var("f"), Term::var("x"))),
        ),
    );
    let term = Term::app(church_2, id);
    assert_eq!(reduce_to_normal(term), Term::abs("x", Term::var("x")));
}

#[test]
fn bruijn_ao_full_church_2_applied_to_id() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(Term::var("f"), Term::app(Term::var("f"), Term::var("x"))),
        ),
    );
    let term = Term::app(church_2, id);
    assert_eq!(bruijn_reduce_to_normal(term), Term::abs("x", Term::var("x")));
}
