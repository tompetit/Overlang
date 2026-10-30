import os, sys
sys.path.insert(0, os.environ["BB_APP"])
import main

assert main.db_path_from(["9000", "other.db"]) == "other.db"
assert main.db_path_from(["9000"]) == "todo.db"
assert main.db_path_from([]) == "todo.db"
assert main.db_path_from(["9000", "/a/b.db", "extra"]) == "/a/b.db"
print("probe ok")
