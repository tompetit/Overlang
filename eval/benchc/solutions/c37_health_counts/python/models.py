"""Tracker models: records, validators, and every query (spec section 3.2).

Port of examples/tracker/src/02_models.ov, same names and SQL. Standard library only.
Tests: python3 -I -m unittest discover -p models.py
"""
from db import *

import re
import unittest
from dataclasses import dataclass, replace
from typing import Optional

# prelude trim's whitespace; str.strip() with no argument also strips Unicode spaces like U+00A0.
_WS = " \t\n\r\v\f"


@dataclass(frozen=True)
class User:
    id: int
    name: str


@dataclass(frozen=True)
class Project:
    id: int
    key: str
    name: str


@dataclass(frozen=True)
class Label:
    id: int
    project_id: int
    name: str


@dataclass(frozen=True)
class Issue:
    id: int
    project_id: int
    project_key: str
    number: int
    title: str
    body: str
    status: str
    priority: int
    assignee: str
    labels: list[str]


@dataclass(frozen=True)
class Comment:
    id: int
    issue_id: int
    author: str
    body: str


@dataclass(frozen=True)
class Activity:
    id: int
    issue_id: int
    kind: str
    detail: str


@dataclass(frozen=True)
class Filter:
    status: str
    assignee: str
    label: str
    priority: int
    q: str
    page: int


@dataclass(frozen=True)
class ProjectCounts:
    key: str
    open: int
    closed: int


@dataclass(frozen=True)
class Health:
    users: int
    projects: int
    issues: int
    open: int
    closed: int
    labels: int
    comments: int
    by_project: list[ProjectCounts]


@dataclass(frozen=True)
class IssuePage:
    issues: list[Issue]
    total: int
    page: int
    pages: int


# ---- pure helpers ----

def valid_user_name(s: str) -> bool:
    return re.fullmatch(r"[A-Za-z0-9_.-]{1,32}", s) is not None


def valid_project_key(s: str) -> bool:
    return re.fullmatch(r"[A-Z][A-Z0-9]{1,9}", s) is not None


def valid_label_name(s: str) -> bool:
    return re.fullmatch(r"[a-z0-9-]{1,20}", s) is not None


def valid_status(s: str) -> bool:
    return s == "open" or s == "closed"


def issue_key(i: Issue) -> str:
    return i.project_key + "-" + str(i.number)


def like_pattern(q: str) -> str:
    return "%" + q.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_") + "%"


def page_count(total: int) -> int:
    assert 0 <= total <= 1_000_000_000
    result = max(1, (total + 19) // 20)
    assert result >= 1
    return result


def page_offset(page: int) -> int:
    assert 1 <= page <= 1_000_000
    result = (page - 1) * 20
    assert result >= 0
    return result


def no_filter() -> Filter:
    return Filter(status="", assignee="", label="", priority=0, q="", page=1)


# ---- row parsers (column order of section 1.1) ----

def row_user(r: list[str]) -> User:
    return User(id=cell_int(r, 0), name=cell_str(r, 1))


def row_project(r: list[str]) -> Project:
    return Project(id=cell_int(r, 0), key=cell_str(r, 1), name=cell_str(r, 2))


def row_label(r: list[str]) -> Label:
    return Label(id=cell_int(r, 0), project_id=cell_int(r, 1), name=cell_str(r, 2))


def row_issue(r: list[str]) -> Issue:
    return Issue(id=cell_int(r, 0), project_id=cell_int(r, 1), project_key=cell_str(r, 2), number=cell_int(r, 3),
                 title=cell_str(r, 4), body=cell_str(r, 5), status=cell_str(r, 6), priority=cell_int(r, 7),
                 assignee=cell_str(r, 8), labels=[x for x in cell_str(r, 9).split(",") if x != ""])


def row_comment(r: list[str]) -> Comment:
    return Comment(id=cell_int(r, 0), issue_id=cell_int(r, 1), author=cell_str(r, 2), body=cell_str(r, 3))


def row_activity(r: list[str]) -> Activity:
    return Activity(id=cell_int(r, 0), issue_id=cell_int(r, 1), kind=cell_str(r, 2), detail=cell_str(r, 3))


# ---- SQL fragments ----

def issue_select_sql() -> str:
    return ("select i.id, i.project_id, p.key, i.number, i.title, i.body, i.status, i.priority,"
            " coalesce(u.name, ''),"
            " coalesce((select group_concat(l.name, ',' order by l.name) from issue_labels il"
            " join labels l on l.id = il.label_id where il.issue_id = i.id), '')"
            " from issues i join projects p on p.id = i.project_id left join users u on u.id = i.assignee_id")


def issue_where(f: Filter) -> tuple[str, list[str]]:
    sql, ps = "", []
    if f.status != "":
        sql += " and i.status = ?"
        ps.append(p_str(f.status))
    if f.assignee != "":
        sql += " and u.name = ?"
        ps.append(p_str(f.assignee))
    if f.label != "":
        sql += (" and exists (select 1 from issue_labels il2 join labels l2 on l2.id = il2.label_id"
                " where il2.issue_id = i.id and l2.name = ?)")
        ps.append(p_str(f.label))
    if f.priority > 0:
        sql += " and i.priority = ?"
        ps.append(p_int(f.priority))
    if f.q != "":
        sql += " and (i.title like ? escape '\\' or i.body like ? escape '\\')"
        ps += [p_str(like_pattern(f.q))] * 2
    return (sql, ps)


# ---- users, projects, labels ----

def create_user(db: Db, name: str) -> int:
    assert valid_user_name(name)
    return db_int(db, "insert into users (name) values (?) returning id", [p_str(name)])


def list_users(db: Db) -> list[User]:
    return [row_user(r) for r in db_rows(db, "select id, name from users order by id", [])]


def find_user(db: Db, name: str) -> Optional[User]:
    rows = db_rows(db, "select id, name from users where name = ?", [p_str(name)])
    return row_user(rows[0]) if rows else None


def create_project(db: Db, key: str, name: str) -> int:
    assert valid_project_key(key) and len(name.strip(_WS)) > 0
    return db_int(db, "insert into projects (key, name) values (?, ?) returning id", [p_str(key), p_str(name)])


def list_projects(db: Db) -> list[Project]:
    return [row_project(r) for r in db_rows(db, "select id, key, name from projects order by key", [])]


def find_project(db: Db, key: str) -> Optional[Project]:
    rows = db_rows(db, "select id, key, name from projects where key = ?", [p_str(key)])
    return row_project(rows[0]) if rows else None


def create_label(db: Db, project_id: int, name: str) -> int:
    assert valid_label_name(name)
    return db_int(db, "insert into labels (project_id, name) values (?, ?) returning id",
                  [p_int(project_id), p_str(name)])


def list_labels(db: Db, project_id: int) -> list[Label]:
    return [row_label(r) for r in
            db_rows(db, "select id, project_id, name from labels where project_id = ? order by name", [p_int(project_id)])]


def find_label(db: Db, project_id: int, name: str) -> Optional[Label]:
    rows = db_rows(db, "select id, project_id, name from labels where project_id = ? and name = ?",
                   [p_int(project_id), p_str(name)])
    return row_label(rows[0]) if rows else None


# ---- issues ----

def create_issue(db: Db, project_id: int, title: str, body: str, priority: int, assignee_id: int, label_ids: list[int]) -> int:
    assert len(title.strip(_WS)) > 0 and 1 <= priority <= 4
    assignee = p_null() if assignee_id == 0 else p_int(assignee_id)
    issue_id = db_int(db, "insert into issues (project_id, number, title, body, priority, assignee_id)"
                          " select ?, coalesce(max(number), 0) + 1, ?, ?, ?, ? from issues where project_id = ?"
                          " returning id",
                      [p_int(project_id), p_str(title), p_str(body), p_int(priority), assignee, p_int(project_id)])
    if issue_id < 0:
        return -1
    for lid in label_ids:
        db_exec(db, "insert or ignore into issue_labels (issue_id, label_id) values (?, ?)", [p_int(issue_id), p_int(lid)])
    log_activity(db, issue_id, "created", "")
    return db_int(db, "select number from issues where id = ?", [p_int(issue_id)])


def find_issue(db: Db, project_id: int, number: int) -> Optional[Issue]:
    rows = db_rows(db, issue_select_sql() + " where i.project_id = ? and i.number = ?", [p_int(project_id), p_int(number)])
    return row_issue(rows[0]) if rows else None


def count_issues(db: Db, project_id: int, f: Filter) -> int:
    where, ps = issue_where(f)
    return db_int(db, "select count(*) from issues i left join users u on u.id = i.assignee_id where i.project_id = ?" + where,
                  [p_int(project_id)] + ps)


def search_issues(db: Db, project_id: int, f: Filter) -> IssuePage:
    assert 1 <= f.page <= 1_000_000
    where, ps = issue_where(f)
    rows = db_rows(db, issue_select_sql() + " where i.project_id = ?" + where + " order by i.number limit 20 offset ?",
                   [p_int(project_id)] + ps + [p_int(page_offset(f.page))])
    total = max(0, count_issues(db, project_id, f))  # -1 (db error) shows as an empty project
    return IssuePage(issues=[row_issue(r) for r in rows], total=total, page=f.page, pages=page_count(total))


def all_issues(db: Db, project_id: int) -> list[Issue]:
    return [row_issue(r) for r in
            db_rows(db, issue_select_sql() + " where i.project_id = ? order by i.number", [p_int(project_id)])]


def edit_issue(db: Db, i: Issue, title: str, body: str, priority: int) -> int:
    assert 1 <= priority <= 4
    return db_exec(db, "update issues set title = ?, body = ?, priority = ? where id = ?",
                   [p_str(title), p_str(body), p_int(priority), p_int(i.id)])


def set_status(db: Db, i: Issue, status: str) -> bool:
    assert valid_status(status)
    if status == i.status:
        return False
    db_exec(db, "update issues set status = ? where id = ?", [p_str(status), p_int(i.id)])
    log_activity(db, i.id, "status", status)
    return True


def set_assignee(db: Db, i: Issue, user: Optional[User]) -> bool:
    name = user.name if user is not None else ""
    if name == i.assignee:
        return False
    db_exec(db, "update issues set assignee_id = ? where id = ?",
            [p_int(user.id) if user is not None else p_null(), p_int(i.id)])
    log_activity(db, i.id, "assigned", name if name != "" else "-")
    return True


def add_label(db: Db, i: Issue, l: Label) -> int:
    return db_exec(db, "insert or ignore into issue_labels (issue_id, label_id) values (?, ?)", [p_int(i.id), p_int(l.id)])


def remove_label(db: Db, i: Issue, l: Label) -> int:
    return db_exec(db, "delete from issue_labels where issue_id = ? and label_id = ?", [p_int(i.id), p_int(l.id)])


# ---- comments and activity ----

def add_comment(db: Db, i: Issue, author: User, body: str) -> int:
    cid = db_int(db, "insert into comments (issue_id, author_id, body) values (?, ?, ?) returning id",
                 [p_int(i.id), p_int(author.id), p_str(body)])
    if cid > 0:
        log_activity(db, i.id, "commented", author.name)
    return cid


def list_comments(db: Db, issue_id: int) -> list[Comment]:
    return [row_comment(r) for r in db_rows(db, "select c.id, c.issue_id, u.name, c.body from comments c"
                                                " join users u on u.id = c.author_id where c.issue_id = ? order by c.id",
                                            [p_int(issue_id)])]


def log_activity(db: Db, issue_id: int, kind: str, detail: str) -> int:
    return db_int(db, "insert into activity (issue_id, kind, detail) values (?, ?, ?) returning id",
                  [p_int(issue_id), p_str(kind), p_str(detail)])


def list_activity(db: Db, issue_id: int) -> list[Activity]:
    return [row_activity(r) for r in
            db_rows(db, "select id, issue_id, kind, detail from activity where issue_id = ? order by id", [p_int(issue_id)])]


def health_counts(db: Db) -> Optional[Health]:
    """All the /health numbers, or None if any query fails (never partial counts)."""
    totals = db_rows(db, "select (select count(*) from users), (select count(*) from projects),"
                         " (select count(*) from issues), (select count(*) from issues where status = 'open'),"
                         " (select count(*) from issues where status = 'closed'), (select count(*) from labels),"
                         " (select count(*) from comments)", [])
    if not totals:
        return None
    t = [cell_int(totals[0], k) for k in range(7)]
    rows = db_rows(db, "select p.key, count(case when i.status = 'open' then 1 end),"
                       " count(case when i.status = 'closed' then 1 end)"
                       " from projects p left join issues i on i.project_id = p.id group by p.id order by p.key", [])
    if len(rows) != t[1]:  # db_rows answers [] on error
        return None
    return Health(*t, by_project=[ProjectCounts(r[0], cell_int(r, 1), cell_int(r, 2)) for r in rows])


# ---- tests (spec 5.2; underscore names so `from models import *` does not re-export them) ----

def _fixture(db: Db) -> tuple[int, int]:
    """Users alice (1) and bob (2), project WEB (1), labels bug and ui; returns (bug_id, ui_id)."""
    create_user(db, "alice")
    create_user(db, "bob")
    create_project(db, "WEB", "Web site")
    return (create_label(db, 1, "bug"), create_label(db, 1, "ui"))


class _TestModels(unittest.TestCase):
    def test_m_validators(self):
        for k, ok in [("WEB", True), ("W3B", True), ("W", False), ("web", False), ("3WB", False), ("ABCDEFGHIJK", False)]:
            self.assertEqual(valid_project_key(k), ok, k)
        for n, ok in [("alice", True), ("a.b-c_1", True), ("", False), ("a b", False), ("a" * 33, False)]:
            self.assertEqual(valid_user_name(n), ok, n)
        self.assertTrue(valid_label_name("bug-1"))
        self.assertFalse(valid_label_name("Bug"))
        self.assertFalse(valid_label_name(""))
        self.assertTrue(valid_status("open"))
        self.assertFalse(valid_status("Open"))

    def test_m_pure_helpers(self):
        self.assertEqual(like_pattern("50%_a\\b"), "%50\\%\\_a\\\\b%")
        self.assertEqual(page_count(0), 1)
        self.assertEqual(page_count(20), 1)
        self.assertEqual(page_count(21), 2)
        self.assertEqual(page_offset(3), 40)
        self.assertEqual(issue_where(no_filter()), ("", []))
        self.assertEqual(issue_where(replace(no_filter(), status="open", priority=2)),
                         (" and i.status = ? and i.priority = ?", ["sopen", "i2"]))

    def test_m_issue_crud(self):
        db = db_open(":memory:")
        bug, ui = _fixture(db)
        self.assertEqual(create_issue(db, 1, "Fix login", "body", 2, 1, [bug, ui]), 1)
        i = find_issue(db, 1, 1)
        self.assertEqual(i, Issue(id=1, project_id=1, project_key="WEB", number=1, title="Fix login", body="body",
                                  status="open", priority=2, assignee="alice", labels=["bug", "ui"]))
        self.assertEqual(create_issue(db, 1, "Second", "", 3, 0, []), 2)
        api = create_project(db, "API", "Api")
        self.assertEqual(create_issue(db, api, "Other", "", 3, 0, []), 1)
        self.assertIsNone(find_issue(db, 1, 99))
        self.assertEqual(issue_key(i), "WEB-1")
        self.assertIsNone(find_user(db, "nobody"))
        self.assertIsNone(find_project(db, "NOPE"))

    def test_m_search(self):
        db = db_open(":memory:")
        bug, _ = _fixture(db)
        for k in range(1, 26):
            create_issue(db, 1, "issue " + str(k), "50% done" if k == 7 else "", 1 + k % 4, 0, [bug] if k <= 5 else [])
        for k in range(1, 4):
            self.assertTrue(set_status(db, find_issue(db, 1, k), "closed"))
        pg = search_issues(db, 1, no_filter())
        self.assertEqual((len(pg.issues), pg.total, pg.pages, pg.issues[0].number), (20, 25, 2, 1))
        pg2 = search_issues(db, 1, replace(no_filter(), page=2))
        self.assertEqual((len(pg2.issues), pg2.issues[0].number), (5, 21))
        for f, total in [(replace(no_filter(), status="closed"), 3), (replace(no_filter(), label="bug"), 5),
                         (replace(no_filter(), priority=1), 6), (replace(no_filter(), q="ISSUE 2"), 7),
                         (replace(no_filter(), assignee="alice"), 0)]:
            self.assertEqual(search_issues(db, 1, f).total, total, f)
            self.assertEqual(count_issues(db, 1, f), total, f)
        self.assertEqual([i.number for i in search_issues(db, 1, replace(no_filter(), q="%")).issues], [7])
        self.assertEqual(len(all_issues(db, 1)), 25)

    def test_m_health_counts(self):
        db = db_open(":memory:")
        self.assertEqual(health_counts(db), Health(0, 0, 0, 0, 0, 0, 0, []))
        bug, _ = _fixture(db)
        create_project(db, "API", "Api")
        create_issue(db, 1, "a", "", 3, 0, [bug])
        create_issue(db, 1, "b", "", 3, 0, [])
        set_status(db, find_issue(db, 1, 2), "closed")
        add_comment(db, find_issue(db, 1, 1), find_user(db, "bob"), "hi")
        self.assertEqual(health_counts(db), Health(2, 2, 2, 1, 1, 2, 1, [ProjectCounts("API", 0, 0), ProjectCounts("WEB", 1, 1)]))
        db_exec(db, "drop table comments", [])
        self.assertIsNone(health_counts(db))
        self.assertIsNone(health_counts(None))

    def test_m_activity(self):
        db = db_open(":memory:")
        bug, _ = _fixture(db)
        create_issue(db, 1, "One", "", 3, 0, [])
        i = find_issue(db, 1, 1)
        self.assertTrue(set_status(db, i, "closed"))
        i2 = find_issue(db, 1, 1)
        self.assertFalse(set_status(db, i2, "closed"))
        self.assertTrue(set_assignee(db, i2, find_user(db, "bob")))
        i3 = find_issue(db, 1, 1)
        self.assertGreater(add_comment(db, i3, find_user(db, "bob"), "hi"), 0)
        self.assertEqual([a.kind + ":" + a.detail for a in list_activity(db, i.id)],
                         ["created:", "status:closed", "assigned:bob", "commented:bob"])
        self.assertEqual(list_comments(db, i.id), [Comment(id=1, issue_id=1, author="bob", body="hi")])
        i4 = find_issue(db, 1, 1)
        self.assertTrue(set_assignee(db, i4, None))
        self.assertEqual(list_activity(db, i.id)[-1].kind + ":" + list_activity(db, i.id)[-1].detail, "assigned:-")
        l = find_label(db, 1, "bug")
        self.assertEqual(add_label(db, i4, l), 1)
        self.assertEqual(add_label(db, i4, l), 0)
        self.assertEqual(remove_label(db, i4, l), 1)
