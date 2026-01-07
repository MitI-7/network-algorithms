from pathlib import Path
import random
from generalized_maximum_flow import solve

random.seed(722)


def random_gain(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if "in.in" not in str(input_file_path):
            continue

        n, m = 0, 0
        edges = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m = map(int, line.strip().split())
                else:
                    f, t, u = line.strip().split()
                    g = random.uniform(0.001, 1.0)
                    edges.append(f"{f} {t} {0} {u} {g:.3f}")

        exact = solve(n, lines=edges)

        o = Path(str(input_file_path).replace(".in.in", ".txt"))
        with o.open("w") as f:
            f.write(f"{n} {m} {exact}\n")
            f.write("\n".join([e for e in edges]))


def main():
    random_gain(Path("AOJ_6_A"))


if __name__ == "__main__":
    main()
