use lambubu::BruijnLevelsTerm;
use lambubu::Term;

fn round_trip(t: Term) -> Term {
    Term::from((*BruijnLevelsTerm::from_open_term(t)).clone())
}

#[test]
fn closed_term_from_agrees_with_open() {
    let id = Term::abs("x", Term::var("x"));
    let via_from = Term::from(BruijnLevelsTerm::from(id.clone()));
    let via_open = Term::from((*BruijnLevelsTerm::from_open_term(id)).clone());
    assert_eq!(via_from, via_open);
}

#[test]
fn round_trip_var() {
    assert_eq!(round_trip(Term::var("x")), Term::var("x"));
}

#[test]
fn round_trip_abs() {
    let t = Term::abs("x", Term::var("x"));
    assert_eq!(round_trip(t.clone()), t);
}

#[test]
fn round_trip_abs_with_free_var() {
    let t = Term::abs("x", Term::var("y"));
    assert_eq!(round_trip(t.clone()), t);
}

#[test]
fn round_trip_apply() {
    let t = Term::app(Term::var("f"), Term::var("x"));
    assert_eq!(round_trip(t.clone()), t);
}

#[test]
fn round_trip_nested() {
    let t = Term::abs(
        "f",
        Term::abs("x", Term::app(Term::var("f"), Term::var("x"))),
    );
    assert_eq!(round_trip(t.clone()), t);
}

#[test]
fn round_trip_church_2_applied_to_id() {
    let id = Term::abs("y", Term::var("y"));
    let church_2 = Term::abs(
        "f",
        Term::abs(
            "x",
            Term::app(Term::var("f"), Term::app(Term::var("f"), Term::var("x"))),
        ),
    );
    let t = Term::app(church_2, id);
    assert_eq!(round_trip(t.clone()), t);
}

#[test]
fn levels_identity() {
    let id = Term::abs("x", Term::var("x"));
    let b = BruijnLevelsTerm::from(id);
    assert_eq!(
        b,
        BruijnLevelsTerm::Abs(
            0,
            std::rc::Rc::new(BruijnLevelsTerm::Var(0, "x".into())),
            "x".into(),
        )
    );
}

#[test]
fn levels_k_combinator() {
    let k = Term::abs("x", Term::abs("y", Term::var("x")));
    let b = BruijnLevelsTerm::from(k);
    assert_eq!(
        b,
        BruijnLevelsTerm::Abs(
            0,
            std::rc::Rc::new(BruijnLevelsTerm::Abs(
                1,
                std::rc::Rc::new(BruijnLevelsTerm::Var(0, "x".into())),
                "y".into(),
            )),
            "x".into(),
        )
    );
}

#[test]
fn open_term_free_var_levels() {
    let t = Term::app(Term::abs("x", Term::var("x")), Term::var("a"));
    let b = BruijnLevelsTerm::from_open_term(t);
    assert_eq!(
        *b,
        BruijnLevelsTerm::Apply(
            std::rc::Rc::new(BruijnLevelsTerm::Abs(
                1,
                std::rc::Rc::new(BruijnLevelsTerm::Var(1, "x".into())),
                "x".into(),
            )),
            std::rc::Rc::new(BruijnLevelsTerm::Var(0, "a".into())),
        )
    );
}
