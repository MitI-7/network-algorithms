from pathlib import Path
import pulp


def solve_generalized_maximum_flow(num_nodes, source, sink, edges, in_edges, out_edges):
    problem = pulp.LpProblem("GeneralizedMaximumFlow", pulp.LpMaximize)
    flow = pulp.LpVariable.dicts("flow", edges, lowBound=0)

    # objective function
    problem += pulp.lpSum(flow[u, sink] * edges[(u, sink)][1] for u in in_edges[sink]) - pulp.lpSum(
        flow[sink, u] for u in out_edges[sink])

    # 容量制約
    for ((u, v), (c, g)) in edges.items():
        problem += flow[(u, v)] <= c

    # 各頂点の入力フローの合計を制約として追加
    for u in range(num_nodes):
        if u == source or u == sink:
            continue

        problem += pulp.lpSum(flow[(a, u)] * edges[(a, u)][1] for a in in_edges[u]) == pulp.lpSum(
            flow[(u, a)] for a in out_edges[u])

    problem.solve(pulp.PULP_CBC_CMD(msg=0))
    return pulp.value(problem.objective)


def solve(num_nodes: int, lines):
    in_edges = [list() for _ in range(num_nodes)]
    out_edges = [list() for _ in range(num_nodes)]
    edges = dict()
    for line in lines:
        f, t, l, u, g = line.strip().split()
        f, t, l, u, g = int(f), int(t), int(l), int(u), float(g)
        out_edges[f].append(t)
        in_edges[t].append(f)
        assert (f, t) not in edges
        edges[(f, t)] = (u, g)

    source = 0
    sink = num_nodes - 1
    ans = solve_generalized_maximum_flow(num_nodes, source, sink, edges, in_edges, out_edges)
    return ans


def read(input_file_path: Path):
    lines = []
    with input_file_path.open("r") as f:
        for i, line in enumerate(f):
            if i == 0:
                num_nodes, num_edges = map(int, line.strip().split())
                continue

            lines.append(line.strip())

    return solve(num_nodes, lines)


def main():
    print(read(Path("AOJ_6_A/00_sample_00.txt")))


if __name__ == "__main__":
    main()
