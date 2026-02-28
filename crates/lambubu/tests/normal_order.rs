use lambubu::BruijnLevelsTerm;
use lambubu::Term;

fn reduce_to_normal(mut t: Term) -> Term {
    while !t.is_normal_form() {
        t = t.reduce_step_normal_order();
    }
    t
}

fn bruijn_step(t: Term) -> Term {
    let b = BruijnLevelsTerm::from_open_term(t);
    Term::from((*b.reduce_step_normal_order()).clone())
}

fn bruijn_reduce_to_normal(t: Term) -> Term {
    let mut b = BruijnLevelsTerm::from_open_term(t);
    loop {
        if b.is_normal_form() {
            break;
        }
        b = b.reduce_step_normal_order();
    }
    Term::from((*b).clone())
}

// (λx.x) a →_no a
#[test]
fn no_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(term.reduce_step_normal_order(), Term::var("a"));
}

#[test]
fn bruijn_no_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(bruijn_step(term), Term::var("a"));
}

// λz.(λx.x) a →_no λz.a  (reduces under λ, unlike CBN/CBV)
#[test]
fn no_reduces_under_lambda() {
    let term = Term::abs(
        "z",
        Term::app(Term::abs("x", Term::var("x")), Term::var("a")),
    );
    assert_eq!(
        term.reduce_step_normal_order(),
        Term::abs("z", Term::var("a"))
    );
}

#[test]
fn bruijn_no_reduces_under_lambda() {
    let term = Term::abs(
        "z",
        Term::app(Term::abs("x", Term::var("x")), Term::var("a")),
    );
    assert_eq!(bruijn_step(term), Term::abs("z", Term::var("a")));
}

// (λx.x x) ((λy.y) a) →_no ((λy.y) a) ((λy.y) a)
// Argument is NOT reduced before substitution (unlike applicative order / CBV).
#[test]
fn no_does_not_reduce_argument_before_substitution() {
    let arg = Term::app(Term::abs("y", Term::var("y")), Term::var("a"));
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        arg.clone(),
    );
    assert_eq!(term.reduce_step_normal_order(), Term::app(arg.clone(), arg));
}

#[test]
fn bruijn_no_does_not_reduce_argument_before_substitution() {
    let arg = Term::app(Term::abs("y", Term::var("y")), Term::var("a"));
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        arg.clone(),
    );
    assert_eq!(bruijn_step(term), Term::app(arg.clone(), arg));
}

// (λx.λy.x) a b →*_no a  (K combinator)
#[test]
fn no_full_k_combinator() {
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
fn bruijn_no_full_k_combinator() {
    let term = Term::app(
        Term::app(
            Term::abs("x", Term::abs("y", Term::var("x"))),
            Term::var("a"),
        ),
        Term::var("b"),
    );
    assert_eq!(bruijn_reduce_to_normal(term), Term::var("a"));
}

// (λf.λx.f (f x)) (λy.y) →*_no λx.x  (church numeral 2 applied to identity)
// Normal order reaches full normal form, unlike CBN.
#[test]
fn no_full_church_2_applied_to_id() {
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
fn bruijn_no_full_church_2_applied_to_id() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(Term::var("f"), Term::app(Term::var("f"), Term::var("x"))),
        ),
    );
    let term = Term::app(church_2, id);
    assert_eq!(
        bruijn_reduce_to_normal(term),
        Term::abs("x", Term::var("x"))
    );
}

// (x) ((\y.y) z)
#[test]
fn no_argument_reduction() {
    let term = Term::app(
        Term::var("x"),
        Term::app(Term::abs("y", Term::var("y")), Term::var("z")),
    );
    assert_eq!(
        reduce_to_normal(term),
        Term::app(Term::var("x"), Term::var("z"))
    );
}

#[test]
fn bruijn_no_argument_reduction() {
    let term = Term::app(
        Term::var("x"),
        Term::app(Term::abs("y", Term::var("y")), Term::var("z")),
    );
    assert_eq!(
        bruijn_reduce_to_normal(term),
        Term::app(Term::var("x"), Term::var("z"))
    );
}
