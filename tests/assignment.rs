// use network_algorithms::assignment_problem::hungarian2::Hungarian;
// use rstest::rstest;
// use std::fs::read_to_string;
// use std::path::PathBuf;
//
// #[rstest]
// fn assignment(#[files("tests/assignment/*/*.txt")] input_file_path: PathBuf) {
//     let (mut num_nodes, mut expected) = (0, 0);
//
//     let mut cost = Vec::new();
//     read_to_string(&input_file_path).unwrap().split('\n').enumerate().for_each(|(i, line)| {
//         let line: Vec<&str> = line.split_whitespace().collect();
//         if i == 0 {
//             (num_nodes, expected) = (line[0].parse().unwrap(), line[1].parse().unwrap());
//         } else {
//             cost.push(line.iter().map(|x| x.parse::<i64>().unwrap()).collect());
//         }
//     });
//
//     let ans = Hungarian::new(num_nodes, cost).solve();
//     assert_eq!(ans, expected);
// }
