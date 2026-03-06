use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::Measurement,
};
use lambubu::{BruijnLevelsTerm, CompoundEnvironment, Term, compile_term};
use lambubu_church::ChurchEnvironment;

fn standard_environment() -> CompoundEnvironment {
    CompoundEnvironment::new(vec![Box::new(ChurchEnvironment)])
}

fn bench_term<'a, M: Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    term_source: &str,
    standard_reducer: fn(Term) -> Term,
    bruijn_reducer: fn(BruijnLevelsTerm) -> BruijnLevelsTerm,
) {
    let term = compile_term(term_source, &standard_environment()).unwrap();
    let bruijned: BruijnLevelsTerm = term.clone().into();

    group.bench_function(format!("{term_source} - Bruijn"), |b| {
        b.iter(|| {
            let mut bruijned = bruijned.clone();
            while !bruijned.is_normal_form() {
                bruijned = bruijn_reducer(bruijned.clone());
            }
        });
    });

    group.bench_function(format!("{term_source} - Standard"), |b| {
        b.iter(|| {
            let mut term = term.clone();
            while !term.is_normal_form() {
                term = standard_reducer(term.clone());
            }
        });
    });
}

fn ao_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Applicative order benchmark");

    bench_term(
        &mut group,
        "(ADD (ADD 2 5) (ADD 10 15))",
        Term::reduce_step_applicative_order,
        BruijnLevelsTerm::reduce_step_applicative_order,
    );
}

fn no_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Normal order benchmark");

    bench_term(
        &mut group,
        "(ADD (ADD 2 5) (ADD 10 15))",
        Term::reduce_step_normal_order,
        BruijnLevelsTerm::reduce_step_normal_order,
    );
}

criterion_group!(benches, ao_bench, no_bench);
criterion_main!(benches);
