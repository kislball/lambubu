use lambubu::Term;

// x[x := y] = y
#[test]
fn subst_var_hit() {
    let result = Term::Var(::std::rc::Rc::from("x")).substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(result, Term::Var(::std::rc::Rc::from("y")));
}

// z[x := y] = z
#[test]
fn subst_var_miss() {
    let result = Term::Var(::std::rc::Rc::from("z")).substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(result, Term::Var(::std::rc::Rc::from("z")));
}

// (λx.x)[x := y] = λx.x
#[test]
fn subst_bound_variable_no_effect() {
    let term = Term::Abs(::std::rc::Rc::from("x"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))));
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(::std::rc::Rc::from("x"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))))
    );
}

// (λy.x)[x := y] = λy'.y
#[test]
fn subst_capture_avoiding() {
    let term = Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))));
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(::std::rc::Rc::from("y'"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))))
    );
}

// (λy.x)[x := y'] = λy.y'
#[test]
fn subst_capture_avoiding_already_primed() {
    let term = Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))));
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y'")));
    assert_eq!(
        result,
        Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y'"))))
    );
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

// (λx.x)[y := z] = λx.x
#[test]
fn subst_irrelevant() {
    let term = Term::Abs(::std::rc::Rc::from("x"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))));
    let result = term.substitute("y", Term::Var(::std::rc::Rc::from("z")));
    assert_eq!(
        result,
        Term::Abs(::std::rc::Rc::from("x"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))))
    );
}

// (λz.x)[x := y] = λz.y
#[test]
fn subst_under_abs_no_capture() {
    let term = Term::Abs(::std::rc::Rc::from("z"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))));
    let result = term.substitute("x", Term::Var(::std::rc::Rc::from("y")));
    assert_eq!(
        result,
        Term::Abs(::std::rc::Rc::from("z"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))))
    );
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
    assert_eq!(result, Term::Apply(::std::rc::Rc::new(with.clone()), ::std::rc::Rc::new(with)));
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

// (λy.x)[x := λy.y] = λy.λy.y
#[test]
fn subst_with_is_abs_clashing_binder() {
    let term = Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("x"))));
    let with = Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(Term::Var(::std::rc::Rc::from("y"))));
    let result = term.substitute("x", with.clone());
    assert_eq!(result, Term::Abs(::std::rc::Rc::from("y"), ::std::rc::Rc::new(with)));
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
