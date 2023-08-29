from pathlib import Path
import re


stderr_log = Path("completed") / "synthesis" / "3" / "stderr.log"

lines = stderr_log.open("r").readlines()

count = 0
for l in lines:
    if re.search(r".*Checking.*true", l):
        count += 1
print(count)
