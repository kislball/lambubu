use lambubu::BruijnLevelsTerm;
use std::rc::Rc;

#[test]
fn test_bruijn_subst_shadowing() {
    let term = Rc::new(BruijnLevelsTerm::Abs(
        0,
        Rc::new(BruijnLevelsTerm::Var(0, "x".into())),
        "x".into(),
    ));

    let replacement = Rc::new(BruijnLevelsTerm::Var(100, "y".into()));
    let result = (*term).clone().substitute(0, replacement.clone());

    match result {
        BruijnLevelsTerm::Abs(lvl, body, _) => {
            assert_eq!(lvl, 0);
            match body.as_ref() {
                BruijnLevelsTerm::Var(v_lvl, _) => {
                    assert_eq!(
                        *v_lvl, 0,
                        "Variable inside Abs(0) should remain 0 (bound), but was changed!"
                    );
                }
                _ => panic!("Expected body to be Var"),
            }
        }
        _ => panic!("Expected result to be Abs"),
    }
}

#[test]
fn test_bruijn_nested_term_shadowing() {
    let inner = Rc::new(BruijnLevelsTerm::Abs(
        0,
        Rc::new(BruijnLevelsTerm::Var(0, "x".into())),
        "x".into(),
    ));

    let body = Rc::new(BruijnLevelsTerm::Apply(
        Rc::new(BruijnLevelsTerm::Var(0, "y".into())),
        inner.clone(),
    ));

    let replacement = Rc::new(BruijnLevelsTerm::Var(99, "z".into()));

    let result = (*body).clone().substitute(0, replacement.clone());

    match result {
        BruijnLevelsTerm::Apply(a, b) => {
            match a.as_ref() {
                BruijnLevelsTerm::Var(l, _) => {
                    assert_eq!(*l, 99, "Outer variable should be substituted")
                }
                _ => panic!("Left side should be Var"),
            }
            match b.as_ref() {
                BruijnLevelsTerm::Abs(l, inner_body, _) => {
                    assert_eq!(*l, 0);
                    match inner_body.as_ref() {
                        BruijnLevelsTerm::Var(l2, _) => {
                            assert_eq!(*l2, 0, "Inner bound variable must NOT be substituted")
                        }
                        _ => panic!("Inner body should be Var"),
                    }
                }
                _ => panic!("Right side should be Abs"),
            }
        }
        _ => panic!("Result should be Apply"),
    }
}
