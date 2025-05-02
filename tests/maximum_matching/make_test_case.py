from pathlib import Path


def library_checker_general_matching(input_directory_path: Path):
    for input_file_path in input_directory_path.iterdir():
        if ".in" not in str(input_file_path):
            continue

        expected_file_path = input_file_path.with_suffix(".out")
        with expected_file_path.open("r") as f:
            expected = f.readline().strip()

        lines = []
        with input_file_path.open("r") as f:
            for i, line in enumerate(f):
                if i == 0:
                    n, m = map(int, line.strip().split())
                    lines.append(f"{n} {m} {expected}")
                else:
                    lines.append(line.strip())

        output_file_path = Path(str(input_file_path).replace(".in", ".txt"))
        with output_file_path.open("w") as f:
            f.write("\n".join(lines))


def main():
    library_checker_general_matching(Path("LibraryChecker_general_matching"))


if __name__ == "__main__":
    main()
