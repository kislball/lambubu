use lambubu::BruijnLevelsTerm;
use lambubu::Term;

fn reduce_to_fixed_point(mut t: Term) -> Term {
    loop {
        let next = t.clone().reduce_step_call_by_name();
        if next == t {
            return t;
        }
        t = next;
    }
}

fn bruijn_step(t: Term) -> Term {
    let b = BruijnLevelsTerm::from_open_term(t);
    Term::from((*b.reduce_step_call_by_name()).clone())
}

fn bruijn_reduce_to_fixed_point(t: Term) -> Term {
    let mut b = BruijnLevelsTerm::from_open_term(t);
    loop {
        let next = b.clone().reduce_step_call_by_name();
        if *next == *b {
            break;
        }
        b = next;
    }
    Term::from((*b).clone())
}

// (λx.x) a →_cbn a
#[test]
fn cbn_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(term.reduce_step_call_by_name(), Term::var("a"));
}

#[test]
fn bruijn_cbn_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(bruijn_step(term), Term::var("a"));
}

// λz.(λx.x) a →_cbn λz.(λx.x) a  (weak: does NOT reduce under λ)
#[test]
fn cbn_does_not_reduce_under_lambda() {
    let inner = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    let term = Term::abs("z", inner.clone());
    assert_eq!(term.reduce_step_call_by_name(), Term::abs("z", inner));
}

#[test]
fn bruijn_cbn_does_not_reduce_under_lambda() {
    let inner = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    let term = Term::abs("z", inner.clone());
    assert_eq!(bruijn_step(term), Term::abs("z", inner));
}

// (λx.x x) ((λy.y) a) →_cbn ((λy.y) a) ((λy.y) a)
// Argument is substituted unreduced, unlike applicative order / call-by-value.
#[test]
fn cbn_does_not_reduce_argument() {
    let arg = Term::app(Term::abs("y", Term::var("y")), Term::var("a"));
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        arg.clone(),
    );
    assert_eq!(term.reduce_step_call_by_name(), Term::app(arg.clone(), arg));
}

#[test]
fn bruijn_cbn_does_not_reduce_argument() {
    let arg = Term::app(Term::abs("y", Term::var("y")), Term::var("a"));
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        arg.clone(),
    );
    assert_eq!(bruijn_step(term), Term::app(arg.clone(), arg));
}

// (λx.λy.x) a b →*_cbn a  (K combinator)
#[test]
fn cbn_full_k_combinator() {
    let term = Term::app(
        Term::app(
            Term::abs("x", Term::abs("y", Term::var("x"))),
            Term::var("a"),
        ),
        Term::var("b"),
    );
    assert_eq!(
        term.reduce_step_call_by_name()
            .reduce_step_call_by_name()
            .reduce_step_call_by_name()
            .reduce_step_call_by_name()
            .reduce_step_call_by_name(),
        Term::var("a")
    );
}

#[test]
fn bruijn_cbn_full_k_combinator() {
    let term = Term::app(
        Term::app(
            Term::abs("x", Term::abs("y", Term::var("x"))),
            Term::var("a"),
        ),
        Term::var("b"),
    );
    assert_eq!(bruijn_reduce_to_fixed_point(term), Term::var("a"));
}

// (λf.λx.f (f x)) (λy.y) →*_cbn λx.(λy.y) ((λy.y) x)
// CBN does not reduce under λ so it stops short of full normal form.
#[test]
fn cbn_stops_at_weak_head_normal_form() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(Term::var("f"), Term::app(Term::var("f"), Term::var("x"))),
        ),
    );
    let term = Term::app(church_2, id.clone());
    let expected = Term::abs("x", Term::app(id.clone(), Term::app(id, Term::var("x"))));
    assert_eq!(reduce_to_fixed_point(term), expected);
}

#[test]
fn bruijn_cbn_stops_at_weak_head_normal_form() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(Term::var("f"), Term::app(Term::var("f"), Term::var("x"))),
        ),
    );
    let term = Term::app(church_2, id.clone());
    let expected = Term::abs("x", Term::app(id.clone(), Term::app(id, Term::var("x"))));
    assert_eq!(bruijn_reduce_to_fixed_point(term), expected);
}
