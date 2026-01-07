import networkx as nx
from pathlib import Path


def make_test_case(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if ".in.in" not in str(input_file_path):
            continue

        print(input_file_path)

        G = nx.DiGraph()
        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m, _r = map(int, line.strip().split())
                    lines.append(f"{n} {m}")
                else:
                    f, t, w = line.strip().split()
                    G.add_edge(int(f), int(t), weight=int(w))
                    lines.append(f"{f} {t} {w}")

        branching = nx.maximum_branching(G)
        total_weight = sum(d['weight'] for u, v, d in branching.edges(data=True))
        lines[0] += f" {total_weight}"
        print("done")

        output_file_path = Path(str(input_file_path).replace(".in.in", ".txt"))
        with output_file_path.open("w", newline="\n") as f:
            f.write("\n".join(lines))


def main():
    # make_test_case(Path("LibraryChecker_directedmst"))
    make_test_case(Path("AOJ_GRL_2_B"))

if __name__ == "__main__":
    main()
