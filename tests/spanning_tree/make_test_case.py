from pathlib import Path

def aoj_grl_2_b(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if ".in.in" not in str(input_file_path):
            continue
        print(input_file_path)

        expected_file_path = input_file_path.with_suffix(".out")
        expected = None
        with expected_file_path.open("r") as f:
            expected = int(f.readline().strip())

        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m, r = map(int, line.strip().split())
                    lines.append(f"{n} {m} {r} {expected}")
                else:
                    f, t, w = line.strip().split()
                    lines.append(f"{f} {t} {w}")

        output_file_path = Path(str(input_file_path).replace(".in.in", ".txt"))
        with output_file_path.open("w", newline="\n") as f:
            f.write("\n".join(lines))


def library_checker_directed_mst(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if ".in" not in str(input_file_path):
            continue
        print(input_file_path)

        expected_file_path = input_file_path.with_suffix(".out")
        expected = None
        with expected_file_path.open("r") as f:
            expected = int(f.readline().strip())

        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m, r = map(int, line.strip().split())
                    lines.append(f"{n} {m} {r} {expected}")
                else:
                    f, t, w = line.strip().split()
                    lines.append(f"{f} {t} {w}")

        output_file_path = Path(str(input_file_path).replace(".in", ".txt"))
        with output_file_path.open("w", newline="\n") as f:
            f.write("\n".join(lines))


def main():
    # library_checker_directed_mst(Path("LibraryChecker_directedmst"))
    aoj_grl_2_b(Path("AOJ_GRL_2_B"))

if __name__ == "__main__":
    main()
