import os, sys
sys.path.insert(0, os.environ["BB_APP"])
import main

assert main.route_get(None, "/nope").status == 404
assert main.route_get(None, "/add").status == 404
assert main.route_post(None, "/nope", "").status == 404
assert main.route_post(None, "/", "").status == 404
assert main.route_post(None, "/todos.txt", "").status == 404
assert main.route_post(None, "/add", "title=  ").status == 400
assert main.route_post(None, "/toggle", "id=0").status == 400
assert main.route_post(None, "/delete", "").status == 400
assert main.route(None, "PUT", "/", "").status == 404
assert main.route(None, "GET", "/nope", "").status == 404
assert main.route(None, "POST", "/toggle", "id=-1").status == 400
print("probe ok")
