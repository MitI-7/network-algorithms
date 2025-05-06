use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use network_algorithms::io::dimacs::read_minimum_cost_flow_dimacs;
use network_algorithms::maximum_flow::FlowNum;
use network_algorithms::minimum_cost_flow::{MinimumCostFlowSolver, PrimalNetworkSimplex};
use network_algorithms::traits::One;
use std::ops::Neg;
use std::str::FromStr;

fn benchmark<Flow>(g: &mut BenchmarkGroup<WallTime>, solver: &mut dyn MinimumCostFlowSolver<Flow>)
where
    Flow: FlowNum + Neg<Output = Flow> + std::ops::Mul<Output = Flow> + One + FromStr + std::fmt::Debug,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let files = vec![
        ("netgen_8_10a", "benches/data/minimum_cost_flow/netgen_8_10a.min", "369269289"),
        ("netgen_8_12a", "benches/data/minimum_cost_flow/netgen_8_12a.min", "783715427"),
        ("netgen_8_14a", "benches/data/minimum_cost_flow/netgen_8_14a.min", "1772056888"),
        ("netgen_8_16a", "benches/data/minimum_cost_flow/netgen_8_16a.min", "4023172764"),
        ("netgen_8_18a", "benches/data/minimum_cost_flow/netgen_8_18a.min", "8724935705"),
        // ("netgen_8_20a", "benches/tools/minimum_cost_flow/netgen_8_18a.min", ""),
        // ("netgen_8_22a", "benches/tools/minimum_cost_flow/netgen_8_18a.min", ""),
    ];

    for (name, path, expected) in files {
        let expected: Flow = expected.parse().unwrap();
        let mut graph = read_minimum_cost_flow_dimacs::<Flow>(black_box(path)).unwrap();
        println!("name:{}, #nodes:{}, #edges:{}", name, graph.num_nodes(), graph.num_edges());
        g.bench_function(format!("name:{}, #nodes:{}, #edges:{}", name, graph.num_nodes(), graph.num_edges()), |b| {
            b.iter(|| {
                graph.reset();
                let actual = solver.solve(&mut graph).unwrap();
                assert_eq!(actual, expected);
            })
        });
    }
}

fn bench_primal_network_simplex(c: &mut Criterion) {
    let mut solver = PrimalNetworkSimplex::<i64>::default();
    let mut group = c.benchmark_group("primal network simplex");
    group.sample_size(10);
    benchmark(&mut group, &mut solver);
    group.finish();
}

criterion_group!(benches, bench_primal_network_simplex);
criterion_main!(benches);
