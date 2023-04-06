#!/usr/bin/env python3

import click
from pathlib import Path
import re


def correlate(asm, kernel):
    assert asm.exists()
    assert kernel.exists()

    asm_lines = asm.open("r").readlines()
    kernel_lines = kernel.open("r").readlines()

    # a map from kernel lines -> the number of asm instructions related to it
    score = {}

    # annotate each line of the assembly source with the corresponding
    # line from the kernel source
    res = []
    for line in asm_lines:
        lineno_matches = re.search(r"@(\d+),", line)
        format_matches = re.search(r"# format ([A-Z][A-Z])", line)
        if lineno_matches is not None:
            lineno = int(lineno_matches.group(1))
            src_line = kernel_lines[lineno - 1].strip()
            res.append(f"{line.rstrip()} ; {src_line}\n")

            # keep a map from kernel lines -> count
            if lineno in score:
                score[lineno] += [line]
            else:
                score[lineno] = [line]

        elif format_matches is not None:
            typ = format_matches.group(1)
            label = ""
            if typ == "FF":
                label = "48-bit 2-slot"
            elif typ == "FA":
                label = "56-bit 2-slot"
            elif typ == "FG":
                label = "56-bit 3-slot (1x 1-bit NOP)"
            elif typ == "FH":
                label = "56-bit 4-slot (2x 1-bit NOP)"
            elif typ in ["FB", "FC", "FD", "FE"]:
                label = "128-bit 4-slot"
            res.append(f"{line.rstrip()} ; {label}\n")
        else:
            res.append(line)

    asm.open("w").writelines(res)

    # annotate each line of kernel source, with the number of times it appears
    # in the assembly source
    res = []
    for idx, line in enumerate(kernel_lines):
        i = idx + 1
        if i in score:
            res.append(f"{line.rstrip()} // {len(score[i])}\n")
        else:
            res.append(line)
    kernel.open("w").writelines(res)


@click.command()
@click.argument("asm_file")
@click.argument("kernel_src")
def main(asm_file, kernel_src):
    asm = Path(asm_file)
    kernel = Path(kernel_src)

    correlate(asm, kernel)


if __name__ == "__main__":
    main()
