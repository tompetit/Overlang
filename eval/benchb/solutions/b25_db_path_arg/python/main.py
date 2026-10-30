"""HTTP todo app backed by SQLite. Run: python3 main.py [port]

Port of examples/todo/main.ov, same functions and routes. Standard library only.
"""
import sqlite3
import sys
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import List, Optional
from urllib.parse import parse_qsl, unquote_plus


@dataclass(frozen=True)
class Todo:
    id: int
    title: str
    done: bool


@dataclass(frozen=True)
class Response:
    status: int
    content_type: str
    body: str


# ---- sqlite helpers (the Overlang version binds these from C) ----
_last_error = ""


def sqlite_open(path: str) -> Optional[sqlite3.Connection]:
    global _last_error
    try:
        return sqlite3.connect(path, isolation_level=None)
    except sqlite3.Error as e:
        _last_error = str(e)
        return None


def sqlite_close(db: sqlite3.Connection) -> None:
    db.close()


def sqlite_error(db: Optional[sqlite3.Connection]) -> str:
    return _last_error


def _exec(db: Optional[sqlite3.Connection], sql: str, params: tuple, script: bool = False) -> int:
    """Rows changed, or -1 on error (a missing handle is an error)."""
    global _last_error
    if db is None:
        return -1
    try:
        before = db.total_changes
        if script:
            db.executescript(sql)
        else:
            db.execute(sql, params)
        return db.total_changes - before
    except sqlite3.Error as e:
        _last_error = str(e)
        return -1


def sqlite_exec(db: Optional[sqlite3.Connection], sql: str) -> int:
    return _exec(db, sql, (), script=True)


def sqlite_exec_1s(db: Optional[sqlite3.Connection], sql: str, p1: str) -> int:
    return _exec(db, sql, (p1,))


def sqlite_exec_2s(db: Optional[sqlite3.Connection], sql: str, p1: str, p2: str) -> int:
    return _exec(db, sql, (p1, p2))


def sqlite_exec_1i(db: Optional[sqlite3.Connection], sql: str, p1: int) -> int:
    return _exec(db, sql, (p1,))


def sqlite_query(db: Optional[sqlite3.Connection], sql: str) -> Optional[str]:
    """Header row of column names, then rows by \\x1e, columns by \\x1f, NULL as ''. None on error."""
    global _last_error
    if db is None:
        return None
    try:
        cur = db.execute(sql)
        header = "\x1f".join(d[0] for d in cur.description or [])
        rows = ["\x1f".join("" if v is None else str(v) for v in r) for r in cur.fetchall()]
        return "\x1e".join([header] + rows)
    except sqlite3.Error as e:
        _last_error = str(e)
        return None


# ---- http helpers (pure) ----
def url_decode(s: str) -> str:
    return unquote_plus(s)


def form_value(body: str, key: str) -> str:
    for k, v in parse_qsl(body, keep_blank_values=True):
        if k == key:
            return v
    return ""


def html_escape(s: str) -> str:
    return (s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
             .replace('"', "&quot;").replace("'", "&#39;"))


# ---- database ----
def db_init(db) -> int:
    return sqlite_exec(db, "create table if not exists todos (id integer primary key autoincrement, title text not null, done integer not null default 0)")


def parse_rows(text: str) -> List[Todo]:
    """Splits the shim encoding (header row, then rows by \\x1e, columns by \\x1f) into todos."""
    out: List[Todo] = []
    for row in text.split("\x1e")[1:]:
        match row.split("\x1f"):
            case [id, title, done]:
                try:
                    n = int(id)
                except ValueError:
                    continue
                if n > 0:
                    out.append(Todo(id=n, title=title, done=done == "1"))
    assert all(t.id > 0 for t in out)
    return out


def list_todos(db) -> List[Todo]:
    text = sqlite_query(db, "select id, title, done from todos order by id")
    return parse_rows(text) if text is not None else []


def add_todo(db, title: str) -> int:
    assert len(title.strip()) > 0
    return sqlite_exec_1s(db, "insert into todos (title) values (?)", title.strip())


def toggle_todo(db, id: int) -> int:
    assert id > 0
    return sqlite_exec_1i(db, "update todos set done = 1 - done where id = ?", id)


def delete_todo(db, id: int) -> int:
    assert id > 0
    return sqlite_exec_1i(db, "delete from todos where id = ?", id)


# ---- views ----
def id_form(action: str, id: int, label: str) -> str:
    return ('<form method="post" action="' + action + '" style="display:inline">'
            + '<input type="hidden" name="id" value="' + str(id) + '">'
            + "<button>" + label + "</button></form>")


def render_todo(t: Todo) -> str:
    mark = "[x] " if t.done else "[ ] "
    return ("<li>" + mark + html_escape(t.title) + " "
            + id_form("/toggle", t.id, "toggle") + " "
            + id_form("/delete", t.id, "delete") + "</li>")


def render_page(todos: List[Todo]) -> str:
    return ('<!doctype html><html><head><meta charset="utf-8"><title>Todos</title></head><body><h1>Todos</h1>'
            + '<form method="post" action="/add"><input name="title" placeholder="What needs doing?">'
            + "<button>add</button></form><ul>"
            + "".join(render_todo(t) for t in todos)
            + "</ul></body></html>")


def render_text(todos: List[Todo]) -> str:
    return "".join(str(t.id) + " " + ("[x] " if t.done else "[ ] ") + t.title + "\n" for t in todos)


# ---- routing ----
def respond_page(db) -> Response:
    return Response(status=200, content_type="text/html; charset=utf-8", body=render_page(list_todos(db)))


def bad_request(msg: str) -> Response:
    return Response(status=400, content_type="text/plain", body=msg + "\n")


def route(db, method: str, path: str, body: str) -> Response:
    try:
        id = int(form_value(body, "id"))
    except ValueError:
        id = 0
    match (method, path):
        case ("GET", "/"):
            return respond_page(db)
        case ("GET", "/todos.txt"):
            return Response(status=200, content_type="text/plain; charset=utf-8", body=render_text(list_todos(db)))
        case ("POST", "/add"):
            if len(form_value(body, "title").strip()) == 0:
                return bad_request("title required")
            add_todo(db, form_value(body, "title"))
            return respond_page(db)
        case ("POST", "/toggle"):
            if id > 0:
                toggle_todo(db, id)
                return respond_page(db)
            return bad_request("bad id")
        case ("POST", "/delete"):
            if id > 0:
                delete_todo(db, id)
                return respond_page(db)
            return bad_request("bad id")
        case _:
            return Response(status=404, content_type="text/plain", body="not found\n")


# ---- server ----
def port_from(argv: List[str]) -> int:
    try:
        return int(argv[0])
    except (IndexError, ValueError):
        return 8080


def db_path_from(argv: List[str]) -> str:
    return argv[1] if len(argv) > 1 else "todo.db"


def http_respond(handler: BaseHTTPRequestHandler, status: int, content_type: str, body: str) -> None:
    data = body.encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(data)))
    handler.send_header("Connection", "close")
    handler.end_headers()
    handler.wfile.write(data)


class Handler(BaseHTTPRequestHandler):
    def handle_one(self) -> None:
        n = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(n).decode("utf-8", errors="replace") if n else ""
        r = route(self.server.db, self.command, self.path.split("?", 1)[0], body)
        http_respond(self, r.status, r.content_type, r.body)

    do_GET = do_POST = do_PUT = do_DELETE = handle_one

    def log_message(self, *args) -> None:
        pass


def serve(db, server: HTTPServer) -> None:
    server.db = db
    server.serve_forever()


def main() -> None:
    port = port_from(sys.argv[1:])
    path = db_path_from(sys.argv[1:])
    db = sqlite_open(path)
    if db is None:
        print("cannot open " + path + ": " + sqlite_error(None), flush=True)
        return
    try:
        server = HTTPServer(("127.0.0.1", port), Handler)
    except OSError:
        print("cannot listen on port " + str(port), flush=True)
        return
    db_init(db)
    print("listening on http://127.0.0.1:" + str(port), flush=True)
    serve(db, server)


if __name__ == "__main__":
    main()
