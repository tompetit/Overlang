import os, sys
sys.path.insert(0, os.environ["BB_APP"])
import main

db = main.sqlite_open(":memory:")
main.db_init(db)
assert main.add_todo(db, "fine") == 1
try:
    main.add_todo(None, "x")
except AssertionError:
    print("probe ok")
else:
    raise SystemExit("add_todo with a bad handle did not raise AssertionError")
