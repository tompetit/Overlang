"""Tracker HTTP routes and server (spec section 3.4). Run: python3 main.py [db_path] [port]

Port of examples/tracker/src/04_routes.ov, same names and routes. Standard library only.
Tests: python3 -I -m unittest test_tracker
"""
from render import *
from db import _parse_int
from models import _WS

import sys
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import Callable, Optional
from urllib.parse import parse_qsl


@dataclass(frozen=True)
class Request:
    method: str
    path: str
    query: str
    body: str


@dataclass(frozen=True)
class Response:
    status: int
    content_type: str
    body: str


# ---- http helpers (pure) ----
def form_value(body: str, key: str) -> str:
    # The C shim works on NUL-terminated strings: a NUL (raw or %00) ends the body, key or value there.
    for k, v in parse_qsl(body.partition("\x00")[0], keep_blank_values=True):
        if k.partition("\x00")[0] == key:
            return v.partition("\x00")[0]
    return ""


def resp_html(body: str) -> Response:
    return Response(200, "text/html; charset=utf-8", body)


def resp_text(body: str) -> Response:
    return Response(200, "text/plain; charset=utf-8", body)


def resp_json(body: str) -> Response:
    return Response(200, "application/json", body)


def resp_csv(body: str) -> Response:
    return Response(200, "text/csv; charset=utf-8", body)


def resp_error(status: int, msg: str) -> Response:
    return Response(status, "text/plain; charset=utf-8", msg + "\n")


def clean_text(s: str) -> str:
    return s.replace("\r\n", "\n").replace("\r", "\n").replace("\x1e", " ").replace("\x1f", " ").strip(_WS)


def clean_line(s: str) -> str:
    return s.replace("\r", " ").replace("\n", " ").replace("\x1e", " ").replace("\x1f", " ").strip(_WS)


def form_line(body: str, key: str) -> str:
    return clean_line(form_value(body, key))


def form_text(body: str, key: str) -> str:
    return clean_text(form_value(body, key))


def parse_nat(s: str) -> int:
    # explicit ASCII check: str.isdigit() also accepts non-ASCII digits
    result = int(s) if 1 <= len(s) <= 9 and all(c in "0123456789" for c in s) else -1
    assert result >= -1
    return result


def issue_ref(seg: str, suffix: str) -> int:
    n = parse_nat(seg[:len(seg) - len(suffix)]) if seg.endswith(suffix) else -1
    result = n if n >= 1 else 0
    assert result >= 0
    return result


def path_segments(path: str) -> list[str]:
    return [s for s in path.split("/") if s != ""]


def parse_filter(query: str) -> tuple[Optional[Filter], str]:
    status = form_line(query, "status")
    if status not in ("", "open", "done"):
        return (None, "bad status")
    pr = form_line(query, "priority")
    priority = 0 if pr == "" else parse_nat(pr)
    if pr != "" and not 1 <= priority <= 4:
        return (None, "bad priority")
    pg = form_line(query, "page")
    page = 1 if pg == "" else parse_nat(pg)
    if not 1 <= page <= 1_000_000:
        return (None, "bad page")
    return (Filter(status=status, assignee=form_line(query, "assignee"), label=form_line(query, "label"),
                   priority=priority, q=form_line(query, "q"), page=page), "")


def parse_label_names(s: str) -> list[str]:
    return [x.strip(_WS) for x in s.split(",") if x.strip(_WS) != ""]


# ---- lookups and shared responses ----
def with_project(db: Db, key: str, f: Callable[[Project], Response]) -> Response:
    p = find_project(db, key)
    return resp_error(404, "no such project: " + key) if p is None else f(p)


def with_issue(db: Db, key: str, n: int, f: Callable[[Project, Issue], Response]) -> Response:
    def go(p: Project) -> Response:
        i = find_issue(db, p.id, n)
        return resp_error(404, "no such issue: " + key + "-" + str(n)) if i is None else f(p, i)
    return with_project(db, key, go)


def respond_issue(db: Db, p: Project, number: int) -> Response:
    return with_issue(db, p.key, number, lambda p, i: resp_html(
        render_issue(i, list_comments(db, i.id), list_activity(db, i.id))))


def respond_project(db: Db, p: Project, f: Filter) -> Response:
    return resp_html(render_project(p, list_labels(db, p.id), f, search_issues(db, p.id, f)))


def _priority(s: str, default: int) -> tuple[int, Optional[Response]]:
    """Form priority ("" means default): the value, or the 400/422 error response."""
    n = default if s == "" else parse_nat(s)
    if n == -1:
        return (0, resp_error(400, "bad priority"))
    if not 1 <= n <= 4:
        return (0, resp_error(422, "bad priority"))
    return (n, None)


# ---- handlers, one per route of section 2.2 ----
def h_home(db: Db) -> Response:
    return resp_html(render_home(list_projects(db)))


def h_health() -> Response:
    return resp_text("ok\n")


def h_users_page(db: Db) -> Response:
    return resp_html(render_users(list_users(db)))


def h_users_text(db: Db) -> Response:
    return resp_text(users_text(list_users(db)))


def h_create_user(db: Db, body: str) -> Response:
    name = form_line(body, "name")
    if name == "":
        return resp_error(422, "name required")
    if not valid_user_name(name):
        return resp_error(422, "bad name")
    if find_user(db, name) is not None:
        return resp_error(422, "user exists")
    create_user(db, name)
    return h_users_page(db)


def h_projects_text(db: Db) -> Response:
    return resp_text(projects_text(list_projects(db)))


def h_create_project(db: Db, body: str) -> Response:
    key = form_line(body, "key")
    name = form_line(body, "name")
    if key == "":
        return resp_error(422, "key required")
    if not valid_project_key(key):
        return resp_error(422, "bad key")
    if name == "":
        return resp_error(422, "name required")
    if find_project(db, key) is not None:
        return resp_error(422, "project exists")
    create_project(db, key, name)
    return with_project(db, key, lambda p: respond_project(db, p, no_filter()))


def h_project_page(db: Db, key: str, query: str) -> Response:
    def go(p: Project) -> Response:
        f, err = parse_filter(query)
        return resp_error(400, err) if f is None else respond_project(db, p, f)
    return with_project(db, key, go)


def h_issues_text(db: Db, key: str, query: str) -> Response:
    def go(p: Project) -> Response:
        f, err = parse_filter(query)
        return resp_error(400, err) if f is None else resp_text(issues_text(search_issues(db, p.id, f).issues))
    return with_project(db, key, go)


def h_issues_csv(db: Db, key: str) -> Response:
    return with_project(db, key, lambda p: resp_csv(issues_csv(all_issues(db, p.id))))


def h_create_issue(db: Db, key: str, body: str) -> Response:
    def go(p: Project) -> Response:
        title = form_line(body, "title")
        if title == "":
            return resp_error(422, "title required")
        priority, err = _priority(form_line(body, "priority"), 3)
        if err is not None:
            return err
        assignee = form_line(body, "assignee")
        user = find_user(db, assignee) if assignee != "" else None
        if assignee != "" and user is None:
            return resp_error(422, "no such user: " + assignee)
        label_ids = []
        for name in parse_label_names(form_line(body, "labels")):
            l = find_label(db, p.id, name)
            if l is None:
                return resp_error(422, "no such label: " + name)
            label_ids.append(l.id)
        number = create_issue(db, p.id, title, form_text(body, "body"), priority, 0 if user is None else user.id, label_ids)
        return respond_issue(db, p, number)
    return with_project(db, key, go)


def h_labels_text(db: Db, key: str) -> Response:
    return with_project(db, key, lambda p: resp_text(labels_text(list_labels(db, p.id))))


def h_create_label(db: Db, key: str, body: str) -> Response:
    def go(p: Project) -> Response:
        name = form_line(body, "name")
        if name == "":
            return resp_error(422, "name required")
        if not valid_label_name(name):
            return resp_error(422, "bad label")
        if find_label(db, p.id, name) is not None:
            return resp_error(422, "label exists")
        create_label(db, p.id, name)
        return respond_project(db, p, no_filter())
    return with_project(db, key, go)


def h_issue_page(db: Db, key: str, n: int) -> Response:
    return with_issue(db, key, n, lambda p, i: respond_issue(db, p, i.number))


def h_issue_json(db: Db, key: str, n: int) -> Response:
    return with_issue(db, key, n, lambda p, i: resp_json(
        issue_json(i, list_comments(db, i.id), list_activity(db, i.id))))


def h_issue_text(db: Db, key: str, n: int) -> Response:
    return with_issue(db, key, n, lambda p, i: resp_text(issue_line(i)))


def h_comments_text(db: Db, key: str, n: int) -> Response:
    return with_issue(db, key, n, lambda p, i: resp_text(comments_text(list_comments(db, i.id))))


def h_activity_text(db: Db, key: str, n: int) -> Response:
    return with_issue(db, key, n, lambda p, i: resp_text(activity_text(list_activity(db, i.id))))


def h_edit_issue(db: Db, key: str, n: int, body: str) -> Response:
    def go(p: Project, i: Issue) -> Response:
        priority, err = _priority(form_line(body, "priority"), i.priority)
        if err is not None:
            return err
        title = form_line(body, "title")
        text = form_text(body, "body")
        edit_issue(db, i, title or i.title, text or i.body, priority)
        return respond_issue(db, p, i.number)
    return with_issue(db, key, n, go)


def h_set_status(db: Db, key: str, n: int, body: str) -> Response:
    def go(p: Project, i: Issue) -> Response:
        status = form_line(body, "status")
        if not valid_status(status):
            return resp_error(422, "bad status")
        set_status(db, i, status)
        return respond_issue(db, p, i.number)
    return with_issue(db, key, n, go)


def h_assign(db: Db, key: str, n: int, body: str) -> Response:
    def go(p: Project, i: Issue) -> Response:
        assignee = form_line(body, "assignee")
        user = find_user(db, assignee) if assignee != "" else None
        if assignee != "" and user is None:
            return resp_error(422, "no such user: " + assignee)
        set_assignee(db, i, user)
        return respond_issue(db, p, i.number)
    return with_issue(db, key, n, go)


def h_add_comment(db: Db, key: str, n: int, body: str) -> Response:
    def go(p: Project, i: Issue) -> Response:
        author = form_line(body, "author")
        if author == "":
            return resp_error(422, "author required")
        user = find_user(db, author)
        if user is None:
            return resp_error(422, "no such user: " + author)
        text = form_text(body, "body")
        if text == "":
            return resp_error(422, "body required")
        add_comment(db, i, user, text)
        return respond_issue(db, p, i.number)
    return with_issue(db, key, n, go)


def _with_label(db: Db, key: str, n: int, body: str, op: Callable[[Db, Issue, Label], int]) -> Response:
    """Routes 23 and 24: same checks, then op (add_label or remove_label)."""
    def go(p: Project, i: Issue) -> Response:
        name = form_line(body, "label")
        if name == "":
            return resp_error(422, "label required")
        l = find_label(db, p.id, name)
        if l is None:
            return resp_error(422, "no such label: " + name)
        op(db, i, l)
        return respond_issue(db, p, i.number)
    return with_issue(db, key, n, go)


def h_add_label(db: Db, key: str, n: int, body: str) -> Response:
    return _with_label(db, key, n, body, add_label)


def h_remove_label(db: Db, key: str, n: int, body: str) -> Response:
    return _with_label(db, key, n, body, remove_label)


# ---- routing ----
def route(db: Db, req: Request) -> Response:
    b = req.body
    match (req.method, path_segments(req.path)):
        case ("GET", []):
            return h_home(db)
        case ("GET", ["health"]):
            return h_health()
        case ("GET", ["users"]):
            return h_users_page(db)
        case ("GET", ["users.txt"]):
            return h_users_text(db)
        case ("POST", ["users"]):
            return h_create_user(db, b)
        case ("GET", ["projects.txt"]):
            return h_projects_text(db)
        case ("POST", ["projects"]):
            return h_create_project(db, b)
        case ("GET", ["p", key]):
            return h_project_page(db, key, req.query)
        case ("GET", ["p", key, "issues.txt"]):
            return h_issues_text(db, key, req.query)
        case ("GET", ["p", key, "issues.csv"]):
            return h_issues_csv(db, key)
        case ("POST", ["p", key, "issues"]):
            return h_create_issue(db, key, b)
        case ("GET", ["p", key, "labels.txt"]):
            return h_labels_text(db, key)
        case ("POST", ["p", key, "labels"]):
            return h_create_label(db, key, b)
        case ("GET", ["p", key, s]) if issue_ref(s, "") > 0:
            return h_issue_page(db, key, issue_ref(s, ""))
        case ("GET", ["p", key, s]) if issue_ref(s, ".json") > 0:
            return h_issue_json(db, key, issue_ref(s, ".json"))
        case ("GET", ["p", key, s]) if issue_ref(s, ".txt") > 0:
            return h_issue_text(db, key, issue_ref(s, ".txt"))
        case ("GET", ["p", key, s, "comments.txt"]) if issue_ref(s, "") > 0:
            return h_comments_text(db, key, issue_ref(s, ""))
        case ("GET", ["p", key, s, "activity.txt"]) if issue_ref(s, "") > 0:
            return h_activity_text(db, key, issue_ref(s, ""))
        case ("POST", ["p", key, s, "edit"]) if issue_ref(s, "") > 0:
            return h_edit_issue(db, key, issue_ref(s, ""), b)
        case ("POST", ["p", key, s, "status"]) if issue_ref(s, "") > 0:
            return h_set_status(db, key, issue_ref(s, ""), b)
        case ("POST", ["p", key, s, "assign"]) if issue_ref(s, "") > 0:
            return h_assign(db, key, issue_ref(s, ""), b)
        case ("POST", ["p", key, s, "comments"]) if issue_ref(s, "") > 0:
            return h_add_comment(db, key, issue_ref(s, ""), b)
        case ("POST", ["p", key, s, "labels"]) if issue_ref(s, "") > 0:
            return h_add_label(db, key, issue_ref(s, ""), b)
        case ("POST", ["p", key, s, "labels", "remove"]) if issue_ref(s, "") > 0:
            return h_remove_label(db, key, issue_ref(s, ""), b)
        case _:
            return resp_error(404, "not found")


# ---- server ----
def parse_args(argv: list[str]) -> tuple[str, int]:
    path = argv[0] if len(argv) >= 1 else "tracker.db"
    port = _parse_int(argv[1], True) if len(argv) >= 2 else None  # prelude parse_int: no spaces or "_"
    return (path, 8080 if port is None else port)


class Handler(BaseHTTPRequestHandler):
    def handle_one(self) -> None:
        n = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(n).decode("utf-8", errors="replace") if n else ""
        path, _, query = self.path.partition("?")
        r = route(self.server.db, Request(self.command, path, query, body))
        data = r.body.encode("utf-8")
        self.send_response(r.status)
        self.send_header("Content-Type", r.content_type)
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(data)

    def __getattr__(self, name: str):
        # Every method reaches route (unknown ones get its 404, as in the C shim), not http.server's 501.
        if name.startswith("do_"):
            return self.handle_one
        raise AttributeError(name)

    def log_message(self, *args) -> None:
        pass


def serve(db: Db, server: HTTPServer) -> None:
    server.db = db
    server.serve_forever()


def main() -> None:
    path, port = parse_args(sys.argv[1:])
    db = db_open(path)
    if db is None:
        print("cannot open " + path, flush=True)
        sys.exit(1)
    try:
        server = HTTPServer(("127.0.0.1", port), Handler)
    except (OSError, OverflowError):
        print("cannot listen on port " + str(port), flush=True)
        sys.exit(1)
    print("listening on http://127.0.0.1:" + str(port), flush=True)
    serve(db, server)


if __name__ == "__main__":
    main()
