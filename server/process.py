import sys
from pathlib import Path
from typing import Iterator, Callable, List
import re
import pprint
from functools import reduce


def matches(regex, fun: Callable) -> Callable:

    def f(input):
        matches = re.search(regex, input)
        if matches is not None:
            return fun(matches)
        else:
            return None

    return f


def process_stderr_log(log):
    with log.open("r") as f:
        for line in f.readlines():
            phrases = [
                "Starting Phase",
                "Initial Program Depth",
                "Final Program Depth",
                "Starting equality saturation",
                "Search time:",
                "Apply time:",
                "Size:",
                "===============",
                "Cost:"
            ]
            # print all the lines that contain a `phrase`
            for p in phrases:
                if p in line:
                    print(line.strip())


def main():
    if len(sys.argv) != 2:
        raise Exception("Incorrect arguments")

    data_dir = Path(sys.argv[1])
    stderr_log = data_dir / "results" / "stderr.log"
    process_stderr_log(stderr_log)


if __name__ == "__main__":
    main()

data_dir = Path("completed/Feb10-1220-mat-mul_4x4_4x4-1-0/")
log = (data_dir / "results" / "stderr.log").open("r").readlines()
log = map(lambda x: x.strip(), log)


class LogFilter:
    def __init__(self):
        pass

    def step(self, log):
        print(len(log))
        return log

    def run(self, log: Iterator[str]):
        res = []
        for datum in self.step(log):
            res.append(datum)
        return res


class Chunker(LogFilter):
    def __init__(self, start, end=None, data: LogFilter = None):
        self.start = start

        # if we have no end, then have it be the function that
        # always returns None
        if end is None:
            self.end = lambda x: None
        else:
            self.end = end

        # the default log filter is the id filter
        if data is None:
            self.data = LogFilter()
        else:
            self.data = data

    def step(self, log):
        chunk = None
        for line in log:
            # the start filter matches
            if self.start(line) is not None:
                # if we have a chunk, then yield it
                if chunk is not None:
                    yield {
                        chunk[0]: self.data.run(chunk[1])
                    }

                # start a new chunk with this line as head
                chunk = (self.start(line), [])

            # if the end filter matches and we have a chunk
            # add the ending line to the chunk, yield it,
            # and then reset it
            elif self.end(line) is not None and chunk is not None:
                chunk[1].append(self.end(line))
                yield {
                    chunk[0]: self.data.run(chunk[1])
                }
                chunk = None

            # if we are working on a chunk, add the current line
            # to that chunk
            elif chunk is not None:
                chunk[1].append(line)

        # if we reach the end of the log and still have a chunk
        # yield it
        if chunk is not None:
            yield {
                chunk[0]: self.data.run(chunk[1])
            }


class LineFilter(LogFilter):
    def __init__(self, regex, f):
        self.regex = regex
        self.f = f

    def step(self, log):
        for line in log:
            matches = re.search(self.regex, line)
            if matches is not None:
                yield self.f(matches)


class Combine(LogFilter):
    def __init__(self, *children: List[LogFilter], post=None):
        self.children = children
        if post is None:
            self.post = lambda x: x
        else:
            self.post = post

    def run(self, log):
        res = []
        for child in self.children:
            res += child.run(log)

        return self.post(res)


egraph_stats = LineFilter(
    r"Size: n=(\d+), e=(\d+)",
    lambda m: {"nodes": int(m.group(1)), "classes": int(m.group(2))}
)

search_time = LineFilter(
    r"Search time: (\d+\.\d+)",
    lambda m: {"search": float(m.group(1))}
)

apply_time = LineFilter(
    r"Apply time: (\d+\.\d+)",
    lambda m: {"apply": float(m.group(1))}
)

phases_f = Chunker(
    start=matches(r"Starting Phase (\w+)", lambda m: m.group(1)),
    data=Chunker(
        start=matches(r"Iteration (\d+)", lambda m: m.group(1)),
        data=Combine(
            search_time,
            egraph_stats,
            apply_time,
            post=lambda res: reduce(lambda a, b: {**a, **b}, res)
        )
    )
)

res = phases_f.run(log)

pp = pprint.PrettyPrinter(indent=2)
pp.pprint(res)
