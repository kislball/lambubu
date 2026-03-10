use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::Measurement,
};
use lambubu::{BruijnLevelsTerm, CompoundEnvironment, Term, compile_term};
use lambubu_church::ChurchEnvironment;

fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}

fn bench_term_order<'a, M: Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    term_source: &str,
    strategy_name: &str,
    standard_reducer: fn(Term) -> (Term, bool),
    bruijn_reducer: fn(BruijnLevelsTerm) -> (BruijnLevelsTerm, bool),
) {
    let term = compile_term(term_source, &standard_environment()).unwrap();
    let bruijned: BruijnLevelsTerm = term.clone().into();

    group.bench_function(format!("{strategy_name} - {term_source} - Bruijn"), |b| {
        b.iter(|| {
            let mut bruijned = bruijned.clone();
            loop {
                let (next, changed) = bruijn_reducer(bruijned);
                bruijned = next;
                if !changed {
                    break;
                }
            }
        });
    });

    group.bench_function(format!("{strategy_name} - {term_source} - Standard"), |b| {
        b.iter(|| {
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

fn bench_term<'a, M: Measurement>(group: &mut BenchmarkGroup<'a, M>, term_source: &str) {
    bench_term_order(
        group,
        term_source,
        "Normal order",
        Term::reduce_step_normal_order,
        BruijnLevelsTerm::reduce_step_normal_order,
    );

    bench_term_order(
        group,
        term_source,
        "Applicative order",
        Term::reduce_step_applicative_order,
        BruijnLevelsTerm::reduce_step_applicative_order,
    );

    bench_term_order(
        group,
        term_source,
        "Call by name",
        Term::reduce_step_call_by_name,
        BruijnLevelsTerm::reduce_step_call_by_name,
    );

    bench_term_order(
        group,
        term_source,
        "Call by value",
        Term::reduce_step_call_by_value,
        BruijnLevelsTerm::reduce_step_call_by_value,
    );
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Term reductions");

    bench_term(&mut group, "(ADD (ADD 2000 5000) (ADD 1000 1500))");
    bench_term(&mut group, "(ADD 1000 3000)");
    bench_term(&mut group, "(6 5)");
}

criterion_group!(benches, bench);
criterion_main!(benches);
