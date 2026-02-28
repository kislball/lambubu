use lambubu::BruijnLevelsTerm;
use lambubu::Term;

fn round_trip(t: Term) -> Term {
    Term::from((*BruijnLevelsTerm::from_open_term(t)).clone())
}

// x[x := y] = y
#[test]
fn subst_var_hit() {
    let result =
        Term::Var(::std::rc::Rc::from("x")).substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(result, Term::Var(::std::rc::Rc::from("y")));
}

#[test]
fn conversion_subst_var_hit() {
    assert_eq!(round_trip(Term::var("x")), Term::var("x"));
    assert_eq!(round_trip(Term::var("y")), Term::var("y"));
}

// z[x := y] = z
#[test]
fn subst_var_miss() {
    let result =
        Term::Var(::std::rc::Rc::from("z")).substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(result, Term::Var(::std::rc::Rc::from("z")));
}

#[test]
fn conversion_subst_var_miss() {
    assert_eq!(round_trip(Term::var("z")), Term::var("z"));
}

// (λx.x)[x := y] = λx.x
#[test]
fn subst_bound_variable_no_effect() {
    let term = Term::Abs(
        ::std::rc::Rc::from("x"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("x"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x")))
        )
    );
}

#[test]
fn conversion_subst_bound_variable_no_effect() {
    let term = Term::abs("x", Term::var("x"));
    assert_eq!(round_trip(term.clone()), term);
}

// (λy.x)[x := y] = λy'.y
#[test]
fn subst_capture_avoiding() {
    let term = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("y'"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y")))
        )
    );
}

#[test]
fn conversion_subst_capture_avoiding() {
    let term = Term::abs("y", Term::var("x"));
    assert_eq!(round_trip(term.clone()), term);
    let result = Term::abs("y'", Term::var("y"));
    assert_eq!(round_trip(result.clone()), result);
}

// (λy.x)[x := y'] = λy.y'
#[test]
fn subst_capture_avoiding_already_primed() {
    let term = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y'")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("y"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y'")))
        )
    );
}

#[test]
fn conversion_subst_capture_avoiding_already_primed() {
    let result = Term::abs("y", Term::var("y'"));
    assert_eq!(round_trip(result.clone()), result);
}

// (λy.x y')[x := y] = λy''.y y'
#[test]
fn subst_capture_avoiding_double_prime() {
    let term = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Apply(
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y'"))),
        )),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("y''"),
            ::std::rc::Rc::new(Term::Apply(
                ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
                ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y'"))),
            )),
        )
    );
}

#[test]
fn conversion_subst_capture_avoiding_double_prime() {
    let input = Term::abs("y", Term::app(Term::var("x"), Term::var("y'")));
    assert_eq!(round_trip(input.clone()), input);
    let result = Term::abs("y''", Term::app(Term::var("y"), Term::var("y'")));
    assert_eq!(round_trip(result.clone()), result);
}

// (λx.x)[y := z] = λx.x
#[test]
fn subst_irrelevant() {
    let term = Term::Abs(
        ::std::rc::Rc::from("x"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let result = term.substitute("y", Term::Var(::std::rc::Rc::from("z")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("x"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x")))
        )
    );
}

#[test]
fn conversion_subst_irrelevant() {
    let term = Term::abs("x", Term::var("x"));
    assert_eq!(round_trip(term.clone()), term);
}

// (λz.x)[x := y] = λz.y
#[test]
fn subst_under_abs_no_capture() {
    let term = Term::Abs(
        ::std::rc::Rc::from("z"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("z"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y")))
        )
    );
}

#[test]
fn conversion_subst_under_abs_no_capture() {
    let term = Term::abs("z", Term::var("x"));
    assert_eq!(round_trip(term.clone()), term);
    let result = Term::abs("z", Term::var("y"));
    assert_eq!(round_trip(result.clone()), result);
}

// (x y)[x := f] = (f y)
#[test]
fn subst_in_apply() {
    let term = Term::Apply(
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("f")));
    assert_eq!(
        result,
        Term::Apply(
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("f"))),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
        )
    );
}

#[test]
fn conversion_subst_in_apply() {
    let term = Term::app(Term::var("x"), Term::var("y"));
    assert_eq!(round_trip(term.clone()), term);
    let result = Term::app(Term::var("f"), Term::var("y"));
    assert_eq!(round_trip(result.clone()), result);
}

// (x x)[x := (f y)] = ((f y) (f y))
#[test]
fn subst_apply_both_sides() {
    let term = Term::Apply(
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let with = Term::Apply(
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("f"))),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
    );
    let result = term.substitute("x", with.clone());
    assert_eq!(
        result,
        Term::Apply(::std::rc::Rc::new(with.clone()), ::std::rc::Rc::new(with))
    );
}

#[test]
fn conversion_subst_apply_both_sides() {
    let result = Term::app(
        Term::app(Term::var("f"), Term::var("y")),
        Term::app(Term::var("f"), Term::var("y")),
    );
    assert_eq!(round_trip(result.clone()), result);
}

// (λy.λx.z)[z := x] = λy.λx'.x
#[test]
fn subst_nested_abs_capture_avoiding() {
    let term = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Abs(
            ::std::rc::Rc::from("x"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("z"))),
        )),
    );
    let result = term.substitute("z", Term::Var(::std::rc::Rc::from("x")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("y"),
            ::std::rc::Rc::new(Term::Abs(
                ::std::rc::Rc::from("x'"),
                ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
            )),
        )
    );
}

#[test]
fn conversion_subst_nested_abs_capture_avoiding() {
    let input = Term::abs("y", Term::abs("x", Term::var("z")));
    assert_eq!(round_trip(input.clone()), input);
    let result = Term::abs("y", Term::abs("x'", Term::var("x")));
    assert_eq!(round_trip(result.clone()), result);
}

// (λy.y x)[x := y] = λy'.y' y
#[test]
fn subst_binder_and_body_both_affected() {
    let term = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Apply(
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
        )),
    );
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(
            ::std::rc::Rc::from("y'"),
            ::std::rc::Rc::new(Term::Apply(
                ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y'"))),
                ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
            )),
        )
    );
}

#[test]
fn conversion_subst_binder_and_body_both_affected() {
    let input = Term::abs("y", Term::app(Term::var("y"), Term::var("x")));
    assert_eq!(round_trip(input.clone()), input);
    let result = Term::abs("y'", Term::app(Term::var("y'"), Term::var("y")));
    assert_eq!(round_trip(result.clone()), result);
}

// (λy.x)[x := λy.y] = λy.λy.y
#[test]
fn subst_with_is_abs_clashing_binder() {
    let term = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
    );
    let with = Term::Abs(
        ::std::rc::Rc::from("y"),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))),
    );
    let result = term.substitute("x", with.clone());
    assert_eq!(
        result,
        Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(with))
    );
}

#[test]
fn conversion_subst_with_is_abs_clashing_binder() {
    let result = Term::abs("y", Term::abs("y", Term::var("y")));
    assert_eq!(round_trip(result.clone()), result);
}

// ((λx.x) z)[z := w] = ((λx.x) w)
#[test]
fn subst_in_apply_with_abs() {
    let term = Term::Apply(
        ::std::rc::Rc::new(Term::Abs(
            ::std::rc::Rc::from("x"),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))),
        )),
        ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("z"))),
    );
    let result = term.substitute("z", Term::Var(::std::rc::Rc::from("w")));
    assert_eq!(
        result,
        Term::Apply(
            ::std::rc::Rc::new(Term::Abs(
                ::std::rc::Rc::from("x"),
                ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x")))
            )),
            ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("w"))),
        )
    );
}

#[test]
fn conversion_subst_in_apply_with_abs() {
    let term = Term::app(Term::abs("x", Term::var("x")), Term::var("z"));
    assert_eq!(round_trip(term.clone()), term);
    let result = Term::app(Term::abs("x", Term::var("x")), Term::var("w"));
    assert_eq!(round_trip(result.clone()), result);
}
