use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::Measurement,
};
use lambubu::{BruijnLevelsTerm, CompoundEnvironment, Term, compile_term};
use lambubu_church::ChurchEnvironment;

fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}

const TERMS: &[&str] = &[
    "(3 4)",
    "(4 3)",
    "(4 4)",
    "(ADD (PRED (SUCC (MULT (ADD 1 2) (PRED 8)))) \
     (SUCC (MULT (ADD (PRED 8) (SUCC 2)) (MULT (ADD 6 4) (ADD 9 8)))))",
    "(MULT (PRED (PRED (MULT (MULT 2 5) (ADD 9 1)))) \
     (ADD (ADD (SUCC (SUCC 6)) (PRED (ADD 2 10))) \
          (SUCC (ADD (ADD 6 9) (MULT 4 7)))))",
    "(MULT (SUCC (SUCC (SUCC (SUCC 4)))) \
     (SUCC (SUCC (MULT (MULT 7 3) (SUCC 5)))))",
    "(IF (EQ 5 (ADD 2 3)) 7 4)",
    "(IF (EQ 5 (ADD 4 3)) 7 4)",
    "(FACT_ITER 5)",
    "(FACT_ITER 3)",
];

fn bench_strategy<'a, M: Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    strategy_name: &str,
    standard_reducer: fn(Term) -> (Term, bool),
    bruijn_reducer: fn(BruijnLevelsTerm) -> (BruijnLevelsTerm, bool),
    per_term: bool,
) {
    let env = standard_environment();
    let terms: Vec<Term> = TERMS
        .iter()
        .map(|t| compile_term(t, &env).unwrap())
        .collect();
    let bruijn_terms: Vec<BruijnLevelsTerm> = terms.iter().cloned().map(Term::into).collect();

    if per_term {
        for (term, term_str) in terms.iter().zip(TERMS.iter()) {
            group.bench_function(format!("{strategy_name} - Standard: {term_str}"), |b| {
                let term = term.clone();
                b.iter(move || {
                    let mut term = term.clone();
                    loop {
                        let (next, changed) = standard_reducer(term);
                        term = next;
                        if !changed {
                            break;
                        }
                    }
                });
            });
        }

        for (term, term_str) in bruijn_terms.iter().zip(TERMS.iter()) {
            group.bench_function(format!("{strategy_name} - Bruijn: {term_str}"), |b| {
                let term = term.clone();
                b.iter(|| {
                    let mut term = term.clone();
                    loop {
                        let (next, changed) = bruijn_reducer(term);
                        term = next;
                        if !changed {
                            break;
                        }
                    }
                });
            });
        }
    } else {
        group.bench_function(format!("{strategy_name} - Standard"), |b| {
            b.iter(|| {
                for mut term in terms.clone() {
                    loop {
                        let (next, changed) = standard_reducer(term);
                        term = next;
                        if !changed {
                            break;
                        }
                    }
                }
            });
        });

        group.bench_function(format!("{strategy_name} - Bruijn"), |b| {
            b.iter(|| {
                for mut term in bruijn_terms.clone() {
                    loop {
                        let (next, changed) = bruijn_reducer(term);
                        term = next;
                        if !changed {
                            break;
                        }
                    }
                }
            });
        });
    }
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Aggregate");
    group.sample_size(25);

    bench_strategy(
        &mut group,
        "Normal order",
        Term::reduce_step_normal_order,
        BruijnLevelsTerm::reduce_step_normal_order,
        false,
    );
    bench_strategy(
        &mut group,
        "Applicative order",
        Term::reduce_step_applicative_order,
        BruijnLevelsTerm::reduce_step_applicative_order,
        false,
    );
    bench_strategy(
        &mut group,
        "Call by name",
        Term::reduce_step_call_by_name,
        BruijnLevelsTerm::reduce_step_call_by_name,
        false,
    );
    bench_strategy(
        &mut group,
        "Call by value",
        Term::reduce_step_call_by_value,
        BruijnLevelsTerm::reduce_step_call_by_value,
        false,
    );
    drop(group);

    let mut group = c.benchmark_group("Individual");
    group.sample_size(40);

    bench_strategy(
        &mut group,
        "Normal order",
        Term::reduce_step_normal_order,
        BruijnLevelsTerm::reduce_step_normal_order,
        true,
    );
    bench_strategy(
        &mut group,
        "Applicative order",
        Term::reduce_step_applicative_order,
        BruijnLevelsTerm::reduce_step_applicative_order,
        true,
    );
    bench_strategy(
        &mut group,
        "Call by name",
        Term::reduce_step_call_by_name,
        BruijnLevelsTerm::reduce_step_call_by_name,
        true,
    );
    bench_strategy(
        &mut group,
        "Call by value",
        Term::reduce_step_call_by_value,
        BruijnLevelsTerm::reduce_step_call_by_value,
        true,
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
