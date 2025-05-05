use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use network_algorithms::io::dimacs::read_maximum_flow_dimacs;
use network_algorithms::maximum_flow::{FlowNum, MaximumFlowSolver, PushRelabelFIFO, PushRelabelHighestLabel};
use std::str::FromStr;

fn benchmark<Flow>(g: &mut BenchmarkGroup<WallTime>, solver: &mut dyn MaximumFlowSolver<Flow>)
where
    Flow: FlowNum + FromStr + std::fmt::Debug,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let files = vec![
        ("1", "benches/tools/maximum_flow/maxflow_test1.dat", "211846"),
        ("2", "benches/tools/maximum_flow/maxflow_test2.dat", "2135"),
        ("3", "benches/tools/maximum_flow/maxflow_test3.dat", "351015"),
        ("4", "benches/tools/maximum_flow/maxflow_test4.dat", "2344"),
    ];

    for (name, path, expected) in files {
        let expected: Flow = expected.parse().unwrap();
        let (mut graph, source, sink) = read_maximum_flow_dimacs::<Flow>(black_box(path)).unwrap();
        g.bench_function(format!("name:{}, #nodes:{}, #edges:{}", name, graph.num_nodes(), graph.num_edges()), |b| {
            b.iter(|| {
                graph.reset();
                let actual = solver.solve(&mut graph, source, sink, None).unwrap();
                assert_eq!(actual, expected);
            })
        });
    }
}

fn bench_push_relabel_fifo(c: &mut Criterion) {
    let mut solver = PushRelabelFIFO::<i64>::default();
    let mut group = c.benchmark_group("Push Relabel FIFO");
    group.sample_size(10);
    benchmark(&mut group, &mut solver);
    group.finish();
}

fn bench_push_relabel_highest_label(c: &mut Criterion) {
    let mut solver = PushRelabelHighestLabel::<i64>::default();
    let mut group = c.benchmark_group("Push Relabel Highest Label");
    group.sample_size(10);
    benchmark(&mut group, &mut solver);
    group.finish();
}

criterion_group!(benches, bench_push_relabel_fifo, bench_push_relabel_highest_label);
criterion_main!(benches);
