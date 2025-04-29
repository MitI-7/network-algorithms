use criterion::{black_box, criterion_group, criterion_main, Criterion};
use network_algorithms::minimum_cost_flow::{CostScalingPushRelabel, Graph, MinimumCostFlowSolver, PrimalNetworkSimplex};
use num_traits::{FromPrimitive, NumAssign, Zero};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Neg;
use std::str::FromStr;

pub fn read_dimacs<Flow>(path: &str) -> Result<Graph<Flow>, Box<dyn std::error::Error>>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + FromStr + Zero + Default,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut graph = Graph::<Flow>::default();

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
                if parts.len() != 4 || parts[1] != "min" {
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
                let node = parts[1].parse::<usize>()? - 1; // DIMACSは1-indexed
                let supply = parts[2].parse::<Flow>()?;
                if supply >= Flow::zero() {
                    graph.add_supply(node, supply);
                } else {
                    graph.add_demand(node, -supply);
                }
            }
            "a" => {
                if parts.len() != 6 {
                    return Err("Invalid arc definition line".into());
                }
                let from = parts[1].parse::<usize>()? - 1;
                let to = parts[2].parse::<usize>()? - 1;
                let lower = parts[3].parse::<Flow>()?;
                let upper = parts[4].parse::<Flow>()?;
                let cost = parts[5].parse::<Flow>()?;
                graph.add_directed_edge(from, to, lower, upper, cost);
            }
            _ => {
                // unknown line type, ignore or error
            }
        }
    }

    Ok(graph)
}

fn benchmark_multiple_graphs<Flow>(c: &mut Criterion, solver: &mut dyn MinimumCostFlowSolver<Flow>)
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + FromStr + Zero + Default + std::fmt::Debug + FromPrimitive,
    <Flow as FromStr>::Err: std::error::Error + 'static,
{
    let mut c = c.benchmark_group("primal_network_simplex");
    c.sample_size(10);

    let files = vec![
        ("netgen_8_14a", "benches/tools/minimum_cost_flow/netgen_8_14a.min", Flow::from_i64(1772056888_i64).unwrap()),
        ("netgen_8_16a", "benches/tools/minimum_cost_flow/netgen_8_16a.min", Flow::from_i64(4023172764_i64).unwrap()),
        ("netgen_8_18a", "benches/tools/minimum_cost_flow/netgen_8_18a.min", Flow::from_i64(8724935705_i64).unwrap()),
    ];

    for (name, path, expected) in files {
        let mut graph = read_dimacs::<Flow>(black_box(path)).unwrap();
        println!("name:{}, #nodes:{}, #edges:{}", name, graph.num_nodes(), graph.num_edges());
        c.bench_function(format!("primal_network_simplex_{}", name), |b| {
            b.iter(|| {
                graph.reset();
                // let mut solver = PrimalNetworkSimplex::new(graph.num_edges());
                // let mut solver = CostScalingPushRelabel::<i64>::default();
                let actual = solver.solve(&mut graph).unwrap();
                assert_eq!(actual, expected);
            })
        });
    }
}

// fn benchmark_cost_primal_network_simplex(c: &mut Criterion) {
//     let mut solver = PrimalNetworkSimplex::CostScalingPushRelabel::<i64>::default();
//     benchmark_multiple_graphs(c, &mut solver);
// }

fn benchmark_cost_scaling_push_relabel(c: &mut Criterion) {
    let mut solver = CostScalingPushRelabel::<i64>::default();
    benchmark_multiple_graphs(c, &mut solver);
}

criterion_group!(benches, benchmark_cost_scaling_push_relabel);
criterion_main!(benches);
