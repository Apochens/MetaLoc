import sys
from datetime import datetime
from git import Repo

LLVM_TRANSFORMS_SCALAR = "llvm/lib/Transforms/Scalar"

if __name__ == "__main__":
    arg = sys.argv[1]
    repo = Repo(arg)

    total_count = 0
    filtered_commits = []

    # iterate all commits in the repo
    for commit in repo.iter_commits():
        date = datetime.fromtimestamp(commit.committed_date)

        # if the commit date is after 2024-01-01, skip it
        if date >= datetime(2024, 1, 1):
            continue

        # print(f"[{date.strftime('%Y-%m-%d')}] {commit.hexsha}")

        commit_files = repo.git.show(commit, name_only=True, format="%n").splitlines()
        has_target_changes = False
        for file in commit_files:
            if str(file).startswith(LLVM_TRANSFORMS_SCALAR) and str(file).endswith(".cpp"):
                has_target_changes = True

        if has_target_changes:
            def filter_addition(s: str) -> bool:
                return s.startswith("+") or s.startswith("-")

            diff_message: str = repo.git.show(commit)
            filtered_message_lines = list(filter(filter_addition, diff_message.splitlines()))

            current_file = ""
            has_update_func = False
            for line in filtered_message_lines:
                if line.startswith(("+++", "---")):
                    current_file = line[6:]
                    continue
                if "setDebugLoc" in line or "applyMergedLocation" in line or "dropLocation" in line:
                    if current_file.startswith(LLVM_TRANSFORMS_SCALAR):
                        has_update_func = True
                        print(f"[{date.strftime('%Y-%m-%d')}] {commit.hexsha[0:7]} {current_file}\n{line}")

            if has_update_func:
                filtered_commits.append(commit)

        total_count += 1

    print(f"Total: {total_count} commits; Filtered: {len(filtered_commits)}")

    # write all the hexsha of the filtered commits to a file
    with open("filtered_commits.txt", "w") as f:
        for commit in filtered_commits:
            f.write(f"{commit.hexsha}\n")   
