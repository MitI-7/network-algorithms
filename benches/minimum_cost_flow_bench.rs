use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use network_algorithms::io::dimacs::read_minimum_cost_flow_dimacs;
use network_algorithms::minimum_cost_flow::{MinimumCostFlowSolver, PrimalNetworkSimplex};
use num_traits::{FromPrimitive, NumAssign, Zero};
use std::ops::Neg;
use std::str::FromStr;

fn benchmark<Flow>(g: &mut BenchmarkGroup<WallTime>, solver: &mut dyn MinimumCostFlowSolver<Flow>)
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + FromStr + Zero + Default + std::fmt::Debug + FromPrimitive,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let files = vec![
        ("netgen_8_10a", "benches/data/minimum_cost_flow/netgen_8_10a.min", Flow::from_i64(369269289_i64).unwrap()),
        ("netgen_8_12a", "benches/data/minimum_cost_flow/netgen_8_12a.min", Flow::from_i64(783715427_i64).unwrap()),
        ("netgen_8_14a", "benches/data/minimum_cost_flow/netgen_8_14a.min", Flow::from_i64(1772056888_i64).unwrap()),
        ("netgen_8_16a", "benches/data/minimum_cost_flow/netgen_8_16a.min", Flow::from_i64(4023172764_i64).unwrap()),
        ("netgen_8_18a", "benches/data/minimum_cost_flow/netgen_8_18a.min", Flow::from_i64(8724935705_i64).unwrap()),
        // ("netgen_8_20a", "benches/tools/minimum_cost_flow/netgen_8_18a.min", Flow::from_i64(8724935705_i64).unwrap()),
        // ("netgen_8_22a", "benches/tools/minimum_cost_flow/netgen_8_18a.min", Flow::from_i64(8724935705_i64).unwrap()),
    ];

    for (name, path, expected) in files {
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
