use crate::BruijnLevelsTerm;
#[cfg(feature = "cache")]
use std::collections::HashMap;
#[cfg(feature = "cache")]
use std::hash::Hash;
use std::rc::Rc;

#[cfg(feature = "cache")]
#[derive(Clone)]
struct ByAddress<T>(Rc<T>);

#[cfg(feature = "cache")]
impl<T> Hash for ByAddress<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.0) as usize).hash(state);
    }
}

#[cfg(feature = "cache")]
impl<T> PartialEq for ByAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

#[cfg(feature = "cache")]
impl<T> Eq for ByAddress<T> {}

#[derive(Default)]
pub struct Reducer {
    #[cfg(feature = "cache")]
    normal_form_cache: HashMap<ByAddress<BruijnLevelsTerm>, Rc<BruijnLevelsTerm>>,
    #[cfg(feature = "cache")]
    is_normal: HashMap<ByAddress<BruijnLevelsTerm>, bool>,
    #[cfg(feature = "cache")]
    whn_form_cache: HashMap<ByAddress<BruijnLevelsTerm>, Rc<BruijnLevelsTerm>>,
    #[cfg(feature = "cache")]
    wn_form_cache: HashMap<ByAddress<BruijnLevelsTerm>, Rc<BruijnLevelsTerm>>,
}

impl Reducer {
    pub fn new() -> Self {
        Self::default()
    }

    fn substitute_inner(
        &self,
        term: Rc<BruijnLevelsTerm>,
        what: usize,
        with: Rc<BruijnLevelsTerm>,
        #[cfg(feature = "cache")] cache: &mut HashMap<
            ByAddress<BruijnLevelsTerm>,
            Rc<BruijnLevelsTerm>,
        >,
    ) -> Rc<BruijnLevelsTerm> {
        #[cfg(feature = "cache")]
        if let Some(res) = cache.get(&ByAddress(term.clone())).cloned() {
            return res;
        }

        let result = match term.as_ref() {
            BruijnLevelsTerm::Var(val, _) if *val == what => with,
            BruijnLevelsTerm::Abs(lvl, body, name) if *lvl != what => {
                let new_body = self.substitute_inner(
                    body.clone(),
                    what,
                    with,
                    #[cfg(feature = "cache")]
                    cache,
                );
                if Rc::ptr_eq(&new_body, body) {
                    term.clone()
                } else {
                    Rc::new(BruijnLevelsTerm::Abs(*lvl, new_body, name.clone()))
                }
            }
            BruijnLevelsTerm::Apply(a, b) => {
                let new_a = self.substitute_inner(
                    a.clone(),
                    what,
                    with.clone(),
                    #[cfg(feature = "cache")]
                    cache,
                );
                let new_b = self.substitute_inner(
                    b.clone(),
                    what,
                    with,
                    #[cfg(feature = "cache")]
                    cache,
                );
                if Rc::ptr_eq(&new_a, a) && Rc::ptr_eq(&new_b, b) {
                    term.clone()
                } else {
                    Rc::new(BruijnLevelsTerm::Apply(new_a, new_b))
                }
            }
            _ => term.clone(),
        };

        #[cfg(feature = "cache")]
        cache.insert(ByAddress(term.clone()), result.clone());
        result
    }

    pub fn is_normal_form(&mut self, term: Rc<BruijnLevelsTerm>) -> bool {
        #[cfg(feature = "cache")]
        if let Some(val) = self.is_normal.get(&ByAddress(term.clone())).copied() {
            return val;
        }

        let result = match term.as_ref() {
            BruijnLevelsTerm::Var(_, _) => true,
            BruijnLevelsTerm::Abs(_, body, _) => self.is_normal_form(body.clone()),
            BruijnLevelsTerm::Apply(t1, t2) => match t1.as_ref() {
                BruijnLevelsTerm::Abs(_, _, _) => false,
                _ => self.is_normal_form(t1.clone()) && self.is_normal_form(t2.clone()),
            },
        };

        #[cfg(feature = "cache")]
        self.is_normal.insert(ByAddress(term), result);
        result
    }

    pub fn substitute(
        &self,
        term: Rc<BruijnLevelsTerm>,
        what: usize,
        with: Rc<BruijnLevelsTerm>,
    ) -> Rc<BruijnLevelsTerm> {
        self.substitute_inner(
            term,
            what,
            with,
            #[cfg(feature = "cache")]
            &mut HashMap::new(),
        )
    }

    pub fn reduce_normal_order(&mut self, term: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        #[cfg(feature = "cache")]
        if let Some(res) = self
            .normal_form_cache
            .get(&ByAddress(term.clone()))
            .cloned()
        {
            return res;
        }

        let result = match term.as_ref() {
            BruijnLevelsTerm::Var(_, _) => term.clone(),
            BruijnLevelsTerm::Abs(depth, body, name) => {
                let new_body = self.reduce_normal_order(body.clone());
                if Rc::ptr_eq(&new_body, body) {
                    term.clone()
                } else {
                    Rc::new(BruijnLevelsTerm::Abs(*depth, new_body, name.clone()))
                }
            }
            BruijnLevelsTerm::Apply(t1, t2) => match t1.as_ref() {
                BruijnLevelsTerm::Abs(depth, body, _) => {
                    self.reduce_normal_order(self.substitute(body.clone(), *depth, t2.clone()))
                }
                _ => {
                    let t1_normal = self.reduce_normal_order(t1.clone());

                    match t1_normal.as_ref() {
                        BruijnLevelsTerm::Abs(depth, body, _) => self
                            .reduce_normal_order(self.substitute(body.clone(), *depth, t2.clone())),
                        _ => {
                            let t2_normal = self.reduce_normal_order(t2.clone());

                            if Rc::ptr_eq(&t1_normal, t1) && Rc::ptr_eq(&t2_normal, t2) {
                                term.clone()
                            } else {
                                Rc::new(BruijnLevelsTerm::Apply(t1_normal, t2_normal))
                            }
                        }
                    }
                }
            },
        };

        #[cfg(feature = "cache")]
        self.normal_form_cache
            .insert(ByAddress(term.clone()), result.clone());
        result
    }

    pub fn reduce_applicative_order(&mut self, term: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        #[cfg(feature = "cache")]
        if let Some(res) = self
            .normal_form_cache
            .get(&ByAddress(term.clone()))
            .cloned()
        {
            return res;
        }

        let result = match term.as_ref() {
            BruijnLevelsTerm::Var(_, _) => term.clone(),
            BruijnLevelsTerm::Abs(depth, body, name) => {
                let new_body = self.reduce_applicative_order(body.clone());
                if Rc::ptr_eq(&new_body, body) {
                    term.clone()
                } else {
                    Rc::new(BruijnLevelsTerm::Abs(*depth, new_body, name.clone()))
                }
            }
            BruijnLevelsTerm::Apply(t1, t2) => {
                let t1_normal = self.reduce_applicative_order(t1.clone());
                let t2_normal = self.reduce_applicative_order(t2.clone());

                if let BruijnLevelsTerm::Abs(depth, body, _) = t1_normal.as_ref() {
                    self.reduce_applicative_order(self.substitute(body.clone(), *depth, t2_normal))
                } else if Rc::ptr_eq(&t1_normal, t1) && Rc::ptr_eq(&t2_normal, t2) {
                    term.clone()
                } else {
                    Rc::new(BruijnLevelsTerm::Apply(t1_normal, t2_normal))
                }
            }
        };

        #[cfg(feature = "cache")]
        self.normal_form_cache
            .insert(ByAddress(term.clone()), result.clone());
        result
    }

    pub fn reduce_call_by_name(&mut self, term: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        #[cfg(feature = "cache")]
        if let Some(res) = self.whn_form_cache.get(&ByAddress(term.clone())).cloned() {
            return res;
        }

        let result = match term.as_ref() {
            BruijnLevelsTerm::Apply(t1, t2) => {
                let t1_cbn = self.reduce_call_by_name(t1.clone());
                match t1_cbn.as_ref() {
                    BruijnLevelsTerm::Abs(depth, body, _) => {
                        self.reduce_call_by_name(self.substitute(body.clone(), *depth, t2.clone()))
                    }
                    _ if Rc::ptr_eq(&t1_cbn, t1) => term.clone(),
                    _ => Rc::new(BruijnLevelsTerm::Apply(t1_cbn, t2.clone())),
                }
            }
            _ => term.clone(),
        };

        #[cfg(feature = "cache")]
        self.whn_form_cache
            .insert(ByAddress(term.clone()), result.clone());
        result
    }

    pub fn reduce_call_by_value(&mut self, term: Rc<BruijnLevelsTerm>) -> Rc<BruijnLevelsTerm> {
        #[cfg(feature = "cache")]
        if let Some(res) = self.wn_form_cache.get(&ByAddress(term.clone())).cloned() {
            return res;
        }

        let result = match term.as_ref() {
            BruijnLevelsTerm::Apply(t1, t2) => {
                let t1_cbn = self.reduce_call_by_value(t1.clone());
                let t2_cbn = self.reduce_call_by_value(t2.clone());

                match t1_cbn.as_ref() {
                    BruijnLevelsTerm::Abs(depth, body, _) => self.reduce_call_by_value(
                        self.substitute(body.clone(), *depth, t2_cbn.clone()),
                    ),
                    _ if Rc::ptr_eq(&t1_cbn, t1) && Rc::ptr_eq(&t2_cbn, t2) => term.clone(),
                    _ => Rc::new(BruijnLevelsTerm::Apply(t1_cbn, t2_cbn)),
                }
            }
            _ => term.clone(),
        };

        #[cfg(feature = "cache")]
        self.wn_form_cache
            .insert(ByAddress(term.clone()), result.clone());
        result
    }
}
