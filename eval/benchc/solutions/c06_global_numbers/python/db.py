"""Tracker db module: SQLite shims with encoded params, schema, row helpers.

Port of examples/tracker/src/01_db.ov (SPEC.md section 3.1). Standard library only.
"""
import re
import sqlite3
import unittest
from typing import Optional

Db = Optional[sqlite3.Connection]

# ---- sqlite shims (the Overlang version binds these from C) ----
_last_error = ""
_INT_MIN, _INT_MAX = -(2 ** 63), 2 ** 63 - 1


def _parse_int(s: str, sign_plus: bool) -> Optional[int]:
    """ASCII decimal int64 like the runtime's parse_int; '+' only when sign_plus."""
    if re.fullmatch(r"[+-]?[0-9]+" if sign_plus else r"-?[0-9]+", s) is None:
        return None
    n = int(s)
    return n if _INT_MIN <= n <= _INT_MAX else None


def _decode_params(params: str) -> Optional[tuple]:
    """The shim's params encoding to a tuple; None if malformed."""
    if params == "":
        return ()
    fields = params.split("\x1f")
    if len(fields) > 32:
        return None
    out: list = []
    for f in fields:
        if f == "n":
            out.append(None)
        elif f.startswith("s"):
            out.append(f[1:])
        elif f.startswith("i") and _parse_int(f[1:], False) is not None:
            out.append(int(f[1:]))
        else:
            return None
    return tuple(out)


def _first_statement(sql: str) -> str:
    """sql up to and including the first ';' that completes a statement."""
    for k, c in enumerate(sql):
        if c == ";" and sqlite3.complete_statement(sql[:k + 1]):
            return sql[:k + 1]
    return sql


def _run(db: Db, sql: str, params: str) -> Optional[sqlite3.Cursor]:
    """Runs the first statement of sql with decoded params; None on error (sets _last_error)."""
    global _last_error
    if db is None:
        _last_error = "no database"
        return None
    ps = _decode_params(params)
    if ps is None:
        _last_error = "bad parameter"
        return None
    stmt = _first_statement(sql)
    try:
        try:
            return db.execute(stmt, ps)
        except sqlite3.ProgrammingError as e:
            # Python demands an exact count; the shim leaves unbound ? as NULL.
            m = re.search(r"uses (\d+), and there are (\d+)", str(e))
            if m is None or int(m.group(1)) < int(m.group(2)):
                raise
            return db.execute(stmt, ps + (None,) * (int(m.group(1)) - int(m.group(2))))
    except (sqlite3.Error, OverflowError) as e:
        _last_error = str(e)
        return None


def sqlite_open(path: str) -> Db:
    global _last_error
    try:
        return sqlite3.connect(path, isolation_level=None)
    except sqlite3.Error as e:
        _last_error = str(e)
        return None


def sqlite_close(db: Db) -> None:
    if db is not None:
        db.close()


def sqlite_error(db: Db) -> str:
    return _last_error


def sqlite_exec(db: Db, sql: str) -> int:
    """Runs a whole script; rows changed, or -1."""
    global _last_error
    if db is None:
        _last_error = "no database"
        return -1
    try:
        before = db.total_changes
        db.executescript(sql)
        return db.total_changes - before
    except sqlite3.Error as e:
        _last_error = str(e)
        return -1


def sqlite_exec_p(db: Db, sql: str, params: str) -> int:
    before = db.total_changes if db is not None else 0
    cur = _run(db, sql, params)
    if cur is None:
        return -1
    cur.fetchall()  # step to completion so the changes are counted
    return db.total_changes - before


def sqlite_query_p(db: Db, sql: str, params: str) -> Optional[str]:
    """Header row of column names, then rows by \\x1e, columns by \\x1f, NULL as ''. None on error."""
    global _last_error
    cur = _run(db, sql, params)
    if cur is None:
        return None
    try:
        rows = cur.fetchall()
    except sqlite3.Error as e:
        _last_error = str(e)
        return None
    header = "\x1f".join(d[0] for d in cur.description or [])
    return "\x1e".join([header] + ["\x1f".join("" if v is None else str(v) for v in r) for r in rows])


# ---- schema ----
def schema_sql() -> str:
    return """create table if not exists users (
  id integer primary key autoincrement,
  name text not null unique
);
create table if not exists projects (
  id integer primary key autoincrement,
  key text not null unique,
  name text not null
);
create table if not exists issues (
  id integer primary key autoincrement,
  project_id integer not null references projects(id),
  number integer not null,
  title text not null,
  body text not null default '',
  status text not null default 'open' check (status in ('open', 'closed')),
  priority integer not null default 3 check (priority between 1 and 4),
  assignee_id integer references users(id),
  unique (number)
);
create table if not exists labels (
  id integer primary key autoincrement,
  project_id integer not null references projects(id),
  name text not null,
  unique (project_id, name)
);
create table if not exists issue_labels (
  issue_id integer not null references issues(id),
  label_id integer not null references labels(id),
  primary key (issue_id, label_id)
);
create table if not exists comments (
  id integer primary key autoincrement,
  issue_id integer not null references issues(id),
  author_id integer not null references users(id),
  body text not null
);
create table if not exists activity (
  id integer primary key autoincrement,
  issue_id integer not null references issues(id),
  kind text not null check (kind in ('created', 'status', 'assigned', 'commented')),
  detail text not null default ''
);
"""


def db_open(path: str) -> Db:
    db = sqlite_open(path)
    if db is not None and sqlite_exec(db, schema_sql()) < 0:
        sqlite_close(db)
        return None
    return db


# ---- params ----
def p_int(n: int) -> str:
    return "i" + str(n)


def p_str(s: str) -> str:
    return "s" + s


def p_null() -> str:
    return "n"


def sql_params(ps: list[str]) -> str:
    return "\x1f".join(ps)


# ---- queries ----
def db_exec(db: Db, sql: str, ps: list[str]) -> int:
    return sqlite_exec_p(db, sql, sql_params(ps))


def db_rows(db: Db, sql: str, ps: list[str]) -> list[list[str]]:
    text = sqlite_query_p(db, sql, sql_params(ps))
    return parse_table(text) if text is not None else []


def db_int(db: Db, sql: str, ps: list[str]) -> int:
    rows = db_rows(db, sql, ps)
    n = _parse_int(cell_str(rows[0], 0), True) if rows else None
    return -1 if n is None else n


def parse_table(text: str) -> list[list[str]]:
    return [row.split("\x1f") for row in text.split("\x1e")[1:]]


def cell_str(row: list[str], i: int) -> str:
    return row[i] if 0 <= i < len(row) else ""


def cell_int(row: list[str], i: int) -> int:
    n = _parse_int(cell_str(row, i), True)
    return 0 if n is None else n


# ---- tests (SPEC 5.1; private so `from db import *` does not export them) ----
class _TestDb(unittest.TestCase):
    def test_db_parse_table(self) -> None:
        self.assertEqual(parse_table("a\x1fb\x1e1\x1fx\x1e2\x1f"), [["1", "x"], ["2", ""]])
        self.assertEqual(parse_table("a\x1fb"), [])
        self.assertEqual(parse_table(""), [])
        self.assertEqual(cell_str(["a"], 3), "")
        self.assertEqual(cell_int(["7", "x"], 0), 7)
        self.assertEqual(cell_int(["7", "x"], 1), 0)

    def test_db_params(self) -> None:
        self.assertEqual(sql_params([p_int(5), p_str("a b"), p_null(), p_int(-2)]), "i5\x1fsa b\x1fn\x1fi-2")
        self.assertEqual(sql_params([]), "")

    def test_db_roundtrip(self) -> None:
        db = db_open(":memory:")
        self.assertIsNotNone(db)
        self.assertEqual(db_exec(db, "insert into users (name) values (?)", [p_str("ann")]), 1)
        self.assertEqual(db_rows(db, "select id, name from users", []), [["1", "ann"]])
        self.assertEqual(db_int(db, "select count(*) from users where name = ?", [p_str("ann")]), 1)
        self.assertEqual(db_rows(db, "select ? is null, ? + 1", [p_null(), p_int(41)]), [["1", "42"]])
        self.assertEqual(db_exec(db, "insert into nope values (1)", []), -1)
        self.assertEqual(db_rows(db, "select * from nope", []), [])
        self.assertEqual(db_exec(db, "select ?", ["x1"]), -1)
        # The rest of the encoding, as the C shim tests check it.
        for bad in ["q1", "i1x", "i1\x1f", "i+1", "n1", "i99999999999999999999", "\x1f".join(["n"] * 33)]:
            self.assertIsNone(sqlite_query_p(db, "select 1", bad), bad)
            self.assertEqual(sqlite_error(db), "bad parameter")
        self.assertEqual(sqlite_query_p(db, "select ?, ?", "i1"), "?\x1f?\x1e1\x1f")
        self.assertIsNone(sqlite_query_p(db, "select ?", "i1\x1fi2"))
        self.assertEqual(db_int(db, "insert into users (name) values (?) returning id", [p_str("bo;b")]), 2)
        self.assertEqual(db_int(db, "select name from users where id = 2", []), -1)
        sqlite_close(db)

    def test_db_schema(self) -> None:
        db = db_open(":memory:")
        self.assertEqual(
            db_rows(db, "select name from sqlite_master where type = 'table' and name != 'sqlite_sequence' order by name", []),
            [["activity"], ["comments"], ["issue_labels"], ["issues"], ["labels"], ["projects"], ["users"]])
        self.assertEqual(sqlite_exec(db, schema_sql()), 0)
        self.assertEqual(sqlite_query_p(db, "select 1; select 2", ""), "1\x1e1")
        self.assertIsNone(db_open("no/such/dir/x.db"))
        sqlite_close(db)
