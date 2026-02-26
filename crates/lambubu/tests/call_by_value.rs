use lambubu::Term;

fn reduce_to_fixed_point(mut t: Term) -> Term {
    loop {
        let next = t.clone().reduce_step_call_by_value();
        if next == t {
            return t;
        }
        t = next;
    }
}

// (λx.x) a →_cbv a
#[test]
fn cbv_basic_beta() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    assert_eq!(term.reduce_step_call_by_value(), Term::var("a"));
}

// λz.(λx.x) a →_cbv λz.(λx.x) a  (weak: does NOT reduce under λ)
#[test]
fn cbv_does_not_reduce_under_lambda() {
    let inner = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    let term = Term::abs("z", inner.clone());
    assert_eq!(term.reduce_step_call_by_value(), Term::abs("z", inner));
}

// (λx.x x) ((λy.y) a) →_cbv (λx.x x) a →_cbv (a a)
// Argument is reduced to a value before substitution, unlike CBN/normal order.
#[test]
fn cbv_reduces_argument_before_substitution() {
    let term = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        Term::app(Term::abs("y", Term::var("y")), Term::var("a")),
    );
    // Step 1: reduce the argument (λy.y) a → a
    let step1 = Term::app(
        Term::abs("x", Term::app(Term::var("x"), Term::var("x"))),
        Term::var("a"),
    );
    assert_eq!(term.reduce_step_call_by_value(), step1.clone());
    // Step 2: apply with the now-value argument
    assert_eq!(
        step1.reduce_step_call_by_value(),
        Term::app(Term::var("a"), Term::var("a"))
    );
}

// (λx.λy.x) a b →*_cbv a  (K combinator)
#[test]
fn cbv_full_k_combinator() {
    let term = Term::app(
        Term::app(
            Term::abs("x", Term::abs("y", Term::var("x"))),
            Term::var("a"),
        ),
        Term::var("b"),
    );
    assert_eq!(reduce_to_fixed_point(term), Term::var("a"));
}

// (λf.λx.f (f x)) (λy.y) →*_cbv λx.(λy.y) ((λy.y) x)
// CBV does not reduce under λ so it stops short of full normal form.
#[test]
fn cbv_stops_at_weak_normal_form() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(
                Term::var("f"),
                Term::app(Term::var("f"), Term::var("x")),
            ),
        ),
    );
    let term = Term::app(church_2, id.clone());
    let expected = Term::abs(
        "x",
        Term::app(
            id.clone(),
            Term::app(id, Term::var("x")),
        ),
    );
    assert_eq!(reduce_to_fixed_point(term), expected);
}
