use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use lambubu::{BruijnLevelsTerm, Term};

fn hash_of(t: &BruijnLevelsTerm) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn levels(t: Term) -> BruijnLevelsTerm {
    BruijnLevelsTerm::from(t)
}

// --- equality ---

#[test]
fn alpha_eq_identity_different_names() {
    // λx.x == λy.y
    assert_eq!(levels(Term::abs("x", Term::var("x"))), levels(Term::abs("y", Term::var("y"))));
}

#[test]
fn alpha_eq_k_combinator_different_names() {
    // λx.λy.x == λa.λb.a
    let lhs = levels(Term::abs("x", Term::abs("y", Term::var("x"))));
    let rhs = levels(Term::abs("a", Term::abs("b", Term::var("a"))));
    assert_eq!(lhs, rhs);
}

#[test]
fn alpha_neq_picks_different_binder() {
    // λx.λy.x != λx.λy.y
    let lhs = levels(Term::abs("x", Term::abs("y", Term::var("x"))));
    let rhs = levels(Term::abs("x", Term::abs("y", Term::var("y"))));
    assert_ne!(lhs, rhs);
}

#[test]
fn alpha_eq_church_2_different_names() {
    // λf.λx.f(f x) == λg.λy.g(g y)
    let mk = |f: &str, x: &str| {
        levels(Term::abs(
            f,
            Term::abs(x, Term::app(Term::var(f), Term::app(Term::var(f), Term::var(x)))),
        ))
    };
    assert_eq!(mk("f", "x"), mk("g", "y"));
}

#[test]
fn alpha_neq_different_structure() {
    // λx.x != λx.λy.x
    let lhs = levels(Term::abs("x", Term::var("x")));
    let rhs = levels(Term::abs("x", Term::abs("y", Term::var("x"))));
    assert_ne!(lhs, rhs);
}

// --- hashing ---

#[test]
fn hash_eq_alpha_equivalent_terms() {
    let a = levels(Term::abs("x", Term::var("x")));
    let b = levels(Term::abs("y", Term::var("y")));
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_eq_k_combinator_different_names() {
    let a = levels(Term::abs("x", Term::abs("y", Term::var("x"))));
    let b = levels(Term::abs("p", Term::abs("q", Term::var("p"))));
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_neq_structurally_different_terms() {
    let a = levels(Term::abs("x", Term::abs("y", Term::var("x"))));
    let b = levels(Term::abs("x", Term::abs("y", Term::var("y"))));
    assert_ne!(hash_of(&a), hash_of(&b));
}

#[test]
fn hashset_deduplicates_alpha_equivalent_terms() {
    let mut set: HashSet<BruijnLevelsTerm> = HashSet::new();
    set.insert(levels(Term::abs("x", Term::var("x"))));
    set.insert(levels(Term::abs("y", Term::var("y"))));
    set.insert(levels(Term::abs("z", Term::var("z"))));
    assert_eq!(set.len(), 1);
}

#[test]
fn hashset_keeps_structurally_distinct_terms() {
    let mut set: HashSet<BruijnLevelsTerm> = HashSet::new();
    set.insert(levels(Term::abs("x", Term::var("x"))));                          // λx.x
    set.insert(levels(Term::abs("x", Term::abs("y", Term::var("x")))));          // λx.λy.x
    set.insert(levels(Term::abs("x", Term::abs("y", Term::var("y")))));          // λx.λy.y
    set.insert(levels(Term::abs("a", Term::var("a"))));                          // duplicate of λx.x
    assert_eq!(set.len(), 3);
}

// --- Term::hash (goes via BruijnLevelsTerm) ---

#[test]
fn term_hash_alpha_equivalent() {
    fn term_hash(t: &Term) -> u64 {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    }
    let a = Term::abs("x", Term::var("x"));
    let b = Term::abs("y", Term::var("y"));
    assert_eq!(term_hash(&a), term_hash(&b));
}
