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
    standard_reducer: fn(Term) -> Term,
    bruijn_reducer: fn(BruijnLevelsTerm) -> BruijnLevelsTerm,
) {
    let term = compile_term(term_source, &standard_environment()).unwrap();
    let bruijned: BruijnLevelsTerm = term.clone().into();

    group.bench_function(format!("{strategy_name} - {term_source} - Bruijn"), |b| {
        b.iter(|| {
            let mut bruijned = bruijned.clone();
            loop {
                let previous = bruijned.clone();
                bruijned = bruijn_reducer(bruijned.clone());
                if previous == bruijned {
                    break;
                }
            }
        });
    });

    group.bench_function(format!("{strategy_name} - {term_source} - Standard"), |b| {
        b.iter(|| {
            let mut term = term.clone();
            loop {
                let previous = term.clone();
                term = standard_reducer(term.clone());
                if previous == term {
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
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Term reductions");

    bench_term(&mut group, "(ADD (ADD 200 500) (ADD 1000 1500))");
}

criterion_group!(benches, bench);
criterion_main!(benches);
