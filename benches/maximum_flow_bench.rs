use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use network_algorithms::maximum_flow::{Graph, MaximumFlowSolver, PushRelabelFIFO, PushRelabelHighestLabel};
use num_traits::{FromPrimitive, NumAssign, Zero};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Neg;
use std::str::FromStr;

pub fn read_dimacs<Flow>(path: &str) -> Result<(Graph<Flow>, usize, usize), Box<dyn std::error::Error>>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + FromStr + Zero + Default,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut graph = Graph::<Flow>::default();
    let mut source = usize::MAX;
    let mut sink = usize::MAX;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "c" => {
                // comment line, ignore
            }
            "p" => {
                if parts.len() != 4 || parts[1] != "max" {
                    return Err("Invalid problem definition line".into());
                }
                let num_nodes = parts[2].parse::<usize>()?;
                let _num_edges = parts[3].parse::<usize>()?;
                graph.add_nodes(num_nodes);
            }
            "n" => {
                if parts.len() != 3 {
                    return Err("Invalid node definition line".into());
                }
                let node = parts[1].parse::<usize>()? - 1;
                if parts[2] == "s" {
                    source = node;
                } else {
                    sink = node;
                }
            }
            "a" => {
                if parts.len() != 4 {
                    return Err("Invalid arc definition line".into());
                }
                let from = parts[1].parse::<usize>()? - 1;
                let to = parts[2].parse::<usize>()? - 1;
                let capacity = parts[3].parse::<Flow>()?;
                graph.add_directed_edge(from, to, capacity);
            }
            _ => {
                // unknown line type, ignore or error
            }
        }
    }

    Ok((graph, source, sink))
}

fn benchmark<Flow>(g: &mut BenchmarkGroup<WallTime>, solver: &mut dyn MaximumFlowSolver<Flow>)
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + FromStr + Zero + Default + FromPrimitive + std::fmt::Debug,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let files = vec![
        ("1", "benches/tools/maximum_flow/maxflow_test1.dat", Flow::from_i64(211846).unwrap()),
        ("2", "benches/tools/maximum_flow/maxflow_test2.dat", Flow::from_i64(2135).unwrap()),
        ("3", "benches/tools/maximum_flow/maxflow_test3.dat", Flow::from_i64(351015).unwrap()),
        ("4", "benches/tools/maximum_flow/maxflow_test4.dat", Flow::from_i64(2344).unwrap()),
    ];

    for (name, path, expected) in files {
        let (mut graph, source, sink) = read_dimacs::<Flow>(black_box(path)).unwrap();
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
