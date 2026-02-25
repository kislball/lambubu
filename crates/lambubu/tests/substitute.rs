use lambubu::Term;

// x[x := y] = y
#[test]
fn subst_var_hit() {
    let result = Term::Var("x".to_owned()).substitute("x", Term::Var("y".to_owned()));
    assert_eq!(result, Term::Var("y".to_owned()));
}

// z[x := y] = z
#[test]
fn subst_var_miss() {
    let result = Term::Var("z".to_owned()).substitute("x", Term::Var("y".to_owned()));
    assert_eq!(result, Term::Var("z".to_owned()));
}

// (λx.x)[x := y] = λx.x
#[test]
fn subst_bound_variable_no_effect() {
    let term = Term::Abs("x".to_owned(), Box::new(Term::Var("x".to_owned())));
    let result = term.substitute("x", Term::Var("y".to_owned()));
    assert_eq!(
        result,
        Term::Abs("x".to_owned(), Box::new(Term::Var("x".to_owned())))
    );
}

// (λy.x)[x := y] = λy'.y
#[test]
fn subst_capture_avoiding() {
    let term = Term::Abs("y".to_owned(), Box::new(Term::Var("x".to_owned())));
    let result = term.substitute("x", Term::Var("y".to_owned()));
    assert_eq!(
        result,
        Term::Abs("y'".to_owned(), Box::new(Term::Var("y".to_owned())))
    );
}

// (λy.x)[x := y'] = λy.y'
#[test]
fn subst_capture_avoiding_already_primed() {
    let term = Term::Abs("y".to_owned(), Box::new(Term::Var("x".to_owned())));
    let result = term.substitute("x", Term::Var("y'".to_owned()));
    assert_eq!(
        result,
        Term::Abs("y".to_owned(), Box::new(Term::Var("y'".to_owned())))
    );
}

// (λy.x y')[x := y] = λy''.y y'
#[test]
fn subst_capture_avoiding_double_prime() {
    let term = Term::Abs(
        "y".to_owned(),
        Box::new(Term::Apply(
            Box::new(Term::Var("x".to_owned())),
            Box::new(Term::Var("y'".to_owned())),
        )),
    );
    let result = term.substitute("x", Term::Var("y".to_owned()));
    assert_eq!(
        result,
        Term::Abs(
            "y''".to_owned(),
            Box::new(Term::Apply(
                Box::new(Term::Var("y".to_owned())),
                Box::new(Term::Var("y'".to_owned())),
            )),
        )
    );
}

// (λx.x)[y := z] = λx.x
#[test]
fn subst_irrelevant() {
    let term = Term::Abs("x".to_owned(), Box::new(Term::Var("x".to_owned())));
    let result = term.substitute("y", Term::Var("z".to_owned()));
    assert_eq!(
        result,
        Term::Abs("x".to_owned(), Box::new(Term::Var("x".to_owned())))
    );
}

// (λz.x)[x := y] = λz.y
#[test]
fn subst_under_abs_no_capture() {
    let term = Term::Abs("z".to_owned(), Box::new(Term::Var("x".to_owned())));
    let result = term.substitute("x", Term::Var("y".to_owned()));
    assert_eq!(
        result,
        Term::Abs("z".to_owned(), Box::new(Term::Var("y".to_owned())))
    );
}

// (x y)[x := f] = (f y)
#[test]
fn subst_in_apply() {
    let term = Term::Apply(
        Box::new(Term::Var("x".to_owned())),
        Box::new(Term::Var("y".to_owned())),
    );
    let result = term.substitute("x", Term::Var("f".to_owned()));
    assert_eq!(
        result,
        Term::Apply(
            Box::new(Term::Var("f".to_owned())),
            Box::new(Term::Var("y".to_owned())),
        )
    );
}

// (x x)[x := (f y)] = ((f y) (f y))
#[test]
fn subst_apply_both_sides() {
    let term = Term::Apply(
        Box::new(Term::Var("x".to_owned())),
        Box::new(Term::Var("x".to_owned())),
    );
    let with = Term::Apply(
        Box::new(Term::Var("f".to_owned())),
        Box::new(Term::Var("y".to_owned())),
    );
    let result = term.substitute("x", with.clone());
    assert_eq!(result, Term::Apply(Box::new(with.clone()), Box::new(with)));
}

// (λy.λx.z)[z := x] = λy.λx'.x
#[test]
fn subst_nested_abs_capture_avoiding() {
    let term = Term::Abs(
        "y".to_owned(),
        Box::new(Term::Abs(
            "x".to_owned(),
            Box::new(Term::Var("z".to_owned())),
        )),
    );
    let result = term.substitute("z", Term::Var("x".to_owned()));
    assert_eq!(
        result,
        Term::Abs(
            "y".to_owned(),
            Box::new(Term::Abs(
                "x'".to_owned(),
                Box::new(Term::Var("x".to_owned())),
            )),
        )
    );
}

// (λy.y x)[x := y] = λy'.y' y
#[test]
fn subst_binder_and_body_both_affected() {
    let term = Term::Abs(
        "y".to_owned(),
        Box::new(Term::Apply(
            Box::new(Term::Var("y".to_owned())),
            Box::new(Term::Var("x".to_owned())),
        )),
    );
    let result = term.substitute("x", Term::Var("y".to_owned()));
    assert_eq!(
        result,
        Term::Abs(
            "y'".to_owned(),
            Box::new(Term::Apply(
                Box::new(Term::Var("y'".to_owned())),
                Box::new(Term::Var("y".to_owned())),
            )),
        )
    );
}

// (λy.x)[x := λy.y] = λy.λy.y
#[test]
fn subst_with_is_abs_clashing_binder() {
    let term = Term::Abs("y".to_owned(), Box::new(Term::Var("x".to_owned())));
    let with = Term::Abs("y".to_owned(), Box::new(Term::Var("y".to_owned())));
    let result = term.substitute("x", with.clone());
    assert_eq!(result, Term::Abs("y".to_owned(), Box::new(with)));
}

// ((λx.x) z)[z := w] = ((λx.x) w)
#[test]
fn subst_in_apply_with_abs() {
    let term = Term::Apply(
        Box::new(Term::Abs(
            "x".to_owned(),
            Box::new(Term::Var("x".to_owned())),
        )),
        Box::new(Term::Var("z".to_owned())),
    );
    let result = term.substitute("z", Term::Var("w".to_owned()));
    assert_eq!(
        result,
        Term::Apply(
            Box::new(Term::Abs(
                "x".to_owned(),
                Box::new(Term::Var("x".to_owned()))
            )),
            Box::new(Term::Var("w".to_owned())),
        )
    );
}
