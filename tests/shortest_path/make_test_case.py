from pathlib import Path


def aoj_grl_1_a(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if "in.in" not in str(input_file_path):
            continue

        expected_file_path = input_file_path.with_suffix(".out")
        expected = []
        with expected_file_path.open("r") as f:
            for line in f:
                expected.append(line.strip())

        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m, source = map(int, line.strip().split())
                    lines.append(f"{n} {m} {source}")
                    lines.append(f" ".join(expected))
                else:
                    f, t, w = line.strip().split()
                    lines.append(f"{f} {t} {w}")

        output_file_path = Path(str(input_file_path).replace(".in.in", ".txt"))
        with output_file_path.open("w", newline="\n") as f:
            f.write("\n".join(lines))



def main():
    aoj_grl_1_a(Path("AOJ_GRL_1_A"))


if __name__ == "__main__":
    main()
