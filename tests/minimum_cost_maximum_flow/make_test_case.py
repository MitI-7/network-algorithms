from pathlib import Path


def aoj_grl_6_b(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if "in.in" not in str(input_file_path):
            continue

        n, m, f = 0, 0, 0
        b = []
        edges = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m, f = map(int, line.strip().split())
                    b = [0] * n
                    (b[0], b[-1]) = (f, -f)
                else:
                    f, t, u, c = line.strip().split()
                    edges.append(f"{f} {t} {0} {u} {c}")

        out = input_file_path.with_suffix(".out")
        with out.open("r") as f:
            exact = int(f.readline().strip())
            if exact == -1:
                exact = "infeasible"

        o = Path(str(input_file_path).replace(".in.in", ".txt"))

        with o.open("w") as f:
            f.write(f"{n} {m} {exact}\n")

            f.write("\n".join([str(x) for x in b]) + "\n")
            f.write("\n".join([e for e in edges]))

        print(input_file_path, out)


def library_checker_min_cost_b_flow(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if ".in" not in str(input_file_path):
            continue

        out = input_file_path.with_suffix(".out")
        with out.open("r") as f:
            exact = f.readline().strip()
            if exact == -1:
                exact = "infeasible"

        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m = map(int, line.strip().split())
                    lines.append(f"{n} {m} {exact}")
                else:
                    lines.append(line.strip())

        o = Path(str(input_file_path).replace(".in", ".txt"))
        with o.open("w") as f:
            f.write("\n".join(lines))


def libre_oj_102(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if ".in" not in str(input_file_path):
            continue

        out = input_file_path.with_suffix(".out")
        with out.open("r") as f:
            exact = f.readline().strip()

        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m = map(int, line.strip().split())
                    lines.append(f"{n} {m} {exact}")
                else:
                    f, t, u, c = map(int, line.strip().split())
                    lines.append(f"{f - 1} {t - 1} {u} {c}")

        o = Path(str(input_file_path).replace(".in", ".txt"))
        with o.open("w") as f:
            f.write("\n".join(lines))


def main():
    # aoj_grl_6_b(Path("AOJ_GRL_6_B"))
    # library_checker_min_cost_b_flow(Path("LibraryChecker_min_cost_b_flow"))
    libre_oj_102(Path("minimum_cost_maximum_flow/LibreOJ_102"))


if __name__ == "__main__":
    main()
