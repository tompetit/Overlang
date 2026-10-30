import os, sys
sys.path.insert(0, os.environ["BB_APP"])
import main

r = main.respond_page([main.Todo(id=7, title="a<b", done=True)])
assert r.status == 200 and "text/html" in r.content_type, r
assert "[x] a&lt;b" in r.body, r.body
assert main.respond_page([]).status == 200
print("probe ok")
