"""Unit tests, port of the Overlang test blocks (SPEC.md section 5). Run: python3 -I -m unittest -v test_tracker

The db, models and render tests live next to their modules; they are re-exported here
under the spec's class names so this one file runs all four.
"""
import unittest

from main import *
from db import _TestDb as TestDb
from models import _TestModels as TestModels
from render import _TestRender as TestRender


def _req(m: str, p: str, q: str, b: str) -> Request:
    return Request(method=m, path=p, query=q, body=b)


class TestRoutes(unittest.TestCase):
    def test_rt_parsing(self):
        self.assertEqual(path_segments("/p/WEB/3/"), ["p", "WEB", "3"])
        self.assertEqual(path_segments("/"), [])
        self.assertEqual(parse_nat("42"), 42)
        for s in ["", "-1", "1x", "1234567890"]:
            self.assertEqual(parse_nat(s), -1, s)
        self.assertEqual(issue_ref("12.json", ".json"), 12)
        self.assertEqual(issue_ref("12", ""), 12)
        self.assertEqual(issue_ref("0", ""), 0)
        self.assertEqual(issue_ref("12.json", ""), 0)
        self.assertEqual(clean_text("a\r\nb\x1fc "), "a\nb c")
        self.assertEqual(clean_line(" a\nb "), "a b")
        self.assertEqual(parse_label_names(" bug, ,ui "), ["bug", "ui"])
        # parity with the C shims (README): ASCII-only trim, NUL ends a form value
        self.assertEqual(clean_line("\xa0a\xa0"), "\xa0a\xa0")
        self.assertEqual(form_value("a=x%00y&b=1", "a"), "x")

    def test_rt_filter(self):
        self.assertEqual(parse_filter("status=open&priority=2&q=a+b&page=3"),
                         (Filter(status="open", assignee="", label="", priority=2, q="a b", page=3), ""))
        self.assertEqual(parse_filter(""), (no_filter(), ""))
        self.assertEqual(parse_filter("status=done"), (None, "bad status"))
        self.assertEqual(parse_filter("priority=5"), (None, "bad priority"))
        self.assertEqual(parse_filter("priority=x"), (None, "bad priority"))
        self.assertEqual(parse_filter("page=0"), (None, "bad page"))

    def test_rt_unknown_404(self):
        self.assertEqual(route(None, _req("GET", "/nope", "", "")),
                         Response(status=404, content_type="text/plain; charset=utf-8", body="not found\n"))
        self.assertEqual(route(None, _req("DELETE", "/", "", "")).status, 404)
        self.assertEqual(route(None, _req("POST", "/users.txt", "", "")).status, 404)
        r = route(None, _req("GET", "/health", "", ""))
        self.assertEqual((r.status, r.body), (200, "ok\n"))

    def test_rt_flow(self):
        db = db_open(":memory:")

        def post(p: str, b: str) -> Response:
            return route(db, _req("POST", p, "", b))

        def get(p: str, q: str = "") -> Response:
            return route(db, _req("GET", p, q, ""))

        r = post("/users", "name=alice")
        self.assertEqual(r.status, 200)
        self.assertIn("<li>alice</li>", r.body)
        self.assertEqual((post("/users", "name=alice").status, post("/users", "name=alice").body), (422, "user exists\n"))
        self.assertEqual((post("/users", "name=").status, post("/users", "name=").body), (422, "name required\n"))
        self.assertEqual((post("/users", "name=a+b").status, post("/users", "name=a+b").body), (422, "bad name\n"))
        r = post("/projects", "key=WEB&name=Web+site")
        self.assertEqual(r.status, 200)
        self.assertIn("<h1>WEB: Web site</h1>", r.body)
        self.assertEqual(post("/p/WEB/labels", "name=bug").status, 200)
        r = post("/p/WEB/issues", "title=Fix+it&labels=bug&assignee=alice&priority=2")
        self.assertEqual(r.status, 200)
        self.assertIn("<h1>WEB-1: Fix it</h1>", r.body)
        self.assertEqual(get("/p/WEB/issues.txt").body, "WEB-1 open P2 alice [bug] Fix it\n")
        self.assertEqual(post("/p/WEB/1/status", "status=closed").status, 200)
        self.assertEqual(get("/p/WEB/1/activity.txt").body, "created\nstatus closed\n")
        r = get("/p/WEB/1.json")
        self.assertEqual(r.content_type, "application/json")
        self.assertTrue(r.body.startswith('{"key":"WEB-1",'))
        self.assertEqual(get("/p/NOPE"), resp_error(404, "no such project: NOPE"))
        self.assertEqual(get("/p/WEB/9"), resp_error(404, "no such issue: WEB-9"))
        self.assertEqual(get("/p/WEB", "page=0"), resp_error(400, "bad page"))
        self.assertEqual(post("/p/WEB/issues", "title=x&priority=9"), resp_error(422, "bad priority"))
        self.assertEqual(post("/p/WEB/issues", "title=x&priority=abc"), resp_error(400, "bad priority"))
        self.assertEqual(post("/p/WEB/issues", "title=x&labels=nope"), resp_error(422, "no such label: nope"))

    def test_rt_args(self):
        self.assertEqual(parse_args([]), ("tracker.db", 8080, False))
        self.assertEqual(parse_args(["x.db"]), ("x.db", 8080, False))
        self.assertEqual(parse_args(["x.db", "9000"]), ("x.db", 9000, False))
        self.assertEqual(parse_args(["x.db", "nine"]), ("x.db", 8080, False))
        self.assertEqual(parse_args(["x.db", " 9000"]), ("x.db", 8080, False))  # prelude parse_int, not int()
        self.assertEqual(parse_args(["x.db", "+9000"]), ("x.db", 9000, False))
        self.assertEqual(parse_args(["--read-only", "x.db", "9000"]), ("x.db", 9000, True))
        self.assertEqual(parse_args(["x.db", "nine", "--read-only"]), ("x.db", 8080, True))

    def test_rt_read_only(self):
        db = db_open(":memory:")
        route(db, _req("POST", "/users", "", "name=alice"))
        ro = lambda m, p, b="": route_read_only(db, _req(m, p, "", b))
        self.assertEqual(ro("POST", "/users", "name=bob"), resp_error(403, "read-only mode"))
        self.assertEqual(ro("POST", "/p/NOPE/1/labels/remove"), resp_error(403, "read-only mode"))
        self.assertEqual(ro("POST", "/p/WEB/abc/status"), resp_error(404, "not found"))
        self.assertEqual(ro("POST", "/users.txt"), resp_error(404, "not found"))
        self.assertEqual(ro("GET", "/health").body, "ok read-only\n")
        self.assertEqual(ro("GET", "/users.txt").body, "1 alice\n")
        self.assertTrue(ro("GET", "/users").body.endswith('<span class="mode">read-only</span></nav>'
                                                         '<h1>Users</h1><ul class="users"><li>alice</li></ul></body></html>'))


if __name__ == "__main__":
    unittest.main()
