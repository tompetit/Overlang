"""Tracker rendering: HTML pages, markdown-lite, text/JSON/CSV formats (spec section 3.3).

Port of examples/tracker/src/03_render.ov, same names. Every function is pure. Standard library only.
Tests: python3 -I -m unittest discover -p render.py
"""
from models import *
from models import _WS

import unittest
from dataclasses import replace
from urllib.parse import quote


def html_escape(s: str) -> str:
    return (s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
             .replace('"', "&quot;").replace("'", "&#39;"))


def url_encode(s: str) -> str:
    return quote(s, safe="")


def json_str(s: str) -> str:
    out = []
    for c in s:
        match c:
            case "\\": out.append("\\\\")
            case '"': out.append('\\"')
            case "\n": out.append("\\n")
            case "\r": out.append("\\r")
            case "\t": out.append("\\t")
            case _: out.append("\\u%04x" % ord(c) if ord(c) < 0x20 else c)
    return '"' + "".join(out) + '"'


def csv_field(s: str) -> str:
    if any(c in s for c in ',"\n\r'):
        return '"' + s.replace('"', '""') + '"'
    return s


def _inline_rest(t: str) -> str:
    """Section 2.5 inline rules over already-escaped text."""
    out = []
    k = 0
    while k < len(t):
        if t[k] == "`" and (e := t.find("`", k + 1)) >= 0:
            out.append("<code>" + t[k + 1:e] + "</code>")
            k = e + 1
            continue
        if t.startswith("**", k) and (e := t.find("**", k + 3)) >= 0:
            out.append("<strong>" + _inline_rest(t[k + 2:e]) + "</strong>")
            k = e + 2
            continue
        if t[k] == "[" and (j := t.find("](", k + 1)) >= 0 and (e := t.find(")", j + 2)) >= 0:
            url = t[j + 2:e]
            if url.startswith(("http://", "https://", "/")):
                out.append('<a href="' + url + '">' + t[k + 1:j] + "</a>")
                k = e + 1
                continue
        out.append(t[k])
        k += 1
    return "".join(out)


def md_inline(s: str) -> str:
    return _inline_rest(html_escape(s))


def md_render(s: str) -> str:
    ls = s.split("\n")
    if ls and ls[-1] == "":
        ls.pop()
    paras: list[str] = []
    run: list[str] = []
    for line in ls + [""]:
        if line.strip(_WS):
            run.append(line)
        elif run:
            paras.append("<p>" + md_inline("\n".join(run)) + "</p>")
            run = []
    return "\n".join(paras)


def html_page(title: str, body: str) -> str:
    return ('<!doctype html><html><head><meta charset="utf-8"><title>' + html_escape(title)
            + '</title></head><body><nav><a href="/">Tracker</a> <a href="/users">Users</a></nav>'
            + body + "</body></html>")


def dash_if_empty(s: str) -> str:
    return "-" if s == "" else s


def labels_display(names: list[str]) -> str:
    return ", ".join(names) if names else "-"


def filter_query(f: Filter, page: int) -> str:
    pairs = [(k, v) for k, v in [("status", f.status), ("assignee", f.assignee), ("label", f.label),
                                 ("priority", str(f.priority) if f.priority > 0 else ""), ("q", f.q)] if v != ""]
    return "&".join([k + "=" + url_encode(v) for k, v in pairs] + ["page=" + str(page)])


def render_home(projects: list[Project]) -> str:
    items = "".join('<li><a href="/p/' + html_escape(p.key) + '">' + html_escape(p.key) + "</a> "
                    + html_escape(p.name) + "</li>" for p in projects)
    return html_page("Tracker", '<h1>Projects</h1><ul class="projects">' + items
                     + '</ul><form method="post" action="/projects"><input name="key" placeholder="KEY"> '
                     '<input name="name" placeholder="name"> <button>create project</button></form>')


def render_users(users: list[User]) -> str:
    items = "".join("<li>" + html_escape(u.name) + "</li>" for u in users)
    return html_page("Users", '<h1>Users</h1><ul class="users">' + items
                     + '</ul><form method="post" action="/users"><input name="name" placeholder="name"> '
                     "<button>create user</button></form>")


def render_issue_row(i: Issue) -> str:
    e = html_escape
    return ('<tr><td><a href="/p/' + e(i.project_key) + "/" + str(i.number) + '">' + e(issue_key(i))
            + "</a></td><td>" + e(i.title) + "</td><td>" + e(i.status) + "</td><td>P" + str(i.priority)
            + "</td><td>" + e(dash_if_empty(i.assignee)) + "</td><td>" + e(labels_display(i.labels)) + "</td></tr>")


def render_pager(p: Project, f: Filter, pg: IssuePage) -> str:
    def link(n: int, text: str) -> str:
        return ' <a href="' + html_escape("/p/" + p.key + "?" + filter_query(f, n)) + '">' + text + "</a>"
    s = '<p class="pager">Page ' + str(pg.page) + " of " + str(pg.pages) + " (" + str(pg.total) + " issues)"
    if pg.page > 1:
        s += link(pg.page - 1, "prev")
    if pg.page < pg.pages:
        s += link(pg.page + 1, "next")
    return s + "</p>"


def render_project(p: Project, labels: list[Label], f: Filter, pg: IssuePage) -> str:
    e = html_escape
    k = e(p.key)
    body = ("<h1>" + k + ": " + e(p.name) + "</h1>"
            + '<form method="get" action="/p/' + k + '"><input name="status" value="' + e(f.status)
            + '" placeholder="status"> <input name="assignee" value="' + e(f.assignee)
            + '" placeholder="assignee"> <input name="label" value="' + e(f.label)
            + '" placeholder="label"> <input name="priority" value="' + (str(f.priority) if f.priority > 0 else "")
            + '" placeholder="priority"> <input name="q" value="' + e(f.q)
            + '" placeholder="search"> <button>filter</button></form>'
            + '<table class="issues"><tr><th>Issue</th><th>Title</th><th>Status</th><th>Priority</th><th>Assignee</th><th>Labels</th></tr>'
            + "".join(render_issue_row(i) for i in pg.issues) + "</table>"
            + render_pager(p, f, pg)
            + '<h2>Labels</h2><p class="labels">' + e(labels_display([l.name for l in labels])) + "</p>"
            + '<form method="post" action="/p/' + k + '/labels"><input name="name" placeholder="label"> <button>add label</button></form>'
            + '<h2>New issue</h2><form method="post" action="/p/' + k + '/issues"><input name="title" placeholder="title"> '
            + '<input name="priority" value="3"> <input name="assignee" placeholder="assignee"> '
            + '<input name="labels" placeholder="labels"> <textarea name="body"></textarea> <button>create issue</button></form>')
    return html_page(p.key + ": " + p.name, body)


def render_issue_meta(i: Issue) -> str:
    e = html_escape
    return ('<p class="meta">Status: <span class="status">' + e(i.status) + "</span> | Priority: P" + str(i.priority)
            + " | Assignee: " + e(dash_if_empty(i.assignee)) + " | Labels: " + e(labels_display(i.labels)) + "</p>")


def render_issue(i: Issue, comments: list[Comment], activity: list[Activity]) -> str:
    e = html_escape
    b = e("/p/" + i.project_key + "/" + str(i.number))
    t, v = ("closed", "close") if i.status == "open" else ("open", "reopen")
    body = ("<h1>" + e(issue_key(i)) + ": " + e(i.title) + "</h1>"
            + render_issue_meta(i)
            + '<div class="body">' + md_render(i.body) + "</div>"
            + '<form method="post" action="' + b + '/status"><input type="hidden" name="status" value="' + t
            + '"><button>' + v + "</button></form>"
            + '<form method="post" action="' + b + '/assign"><input name="assignee" value="' + e(i.assignee)
            + '"> <button>assign</button></form>'
            + '<form method="post" action="' + b + '/labels"><input name="label" placeholder="label"> <button>add label</button></form>'
            + '<form method="post" action="' + b + '/labels/remove"><input name="label" placeholder="label"> <button>remove label</button></form>'
            + '<form method="post" action="' + b + '/edit"><input name="title" value="' + e(i.title)
            + '"> <input name="priority" value="' + str(i.priority) + '"> <textarea name="body">' + e(i.body)
            + "</textarea> <button>save</button></form>"
            + '<h2>Comments</h2><ul class="comments">'
            + "".join("<li><b>" + e(c.author) + "</b> " + md_render(c.body) + "</li>" for c in comments) + "</ul>"
            + '<form method="post" action="' + b + '/comments"><input name="author" placeholder="author"> '
            + '<textarea name="body"></textarea> <button>comment</button></form>'
            + '<h2>Activity</h2><ul class="activity">'
            + "".join("<li>" + e(activity_line(a)) + "</li>" for a in activity) + "</ul>")
    return html_page(issue_key(i) + ": " + i.title, body)


def issue_line(i: Issue) -> str:
    return (issue_key(i) + " " + i.status + " P" + str(i.priority) + " " + dash_if_empty(i.assignee)
            + " [" + ",".join(i.labels) + "] " + i.title + "\n")


def issues_text(xs: list[Issue]) -> str:
    return "".join(issue_line(i) for i in xs)


def users_text(xs: list[User]) -> str:
    return "".join(str(u.id) + " " + u.name + "\n" for u in xs)


def projects_text(xs: list[Project]) -> str:
    return "".join(p.key + " " + p.name + "\n" for p in xs)


def labels_text(xs: list[Label]) -> str:
    return "".join(l.name + "\n" for l in xs)


def comments_text(xs: list[Comment]) -> str:
    return "".join(c.author + ": " + c.body.replace("\n", " ") + "\n" for c in xs)


def activity_line(a: Activity) -> str:
    return a.kind if a.detail == "" else a.kind + " " + a.detail


def activity_text(xs: list[Activity]) -> str:
    return "".join(activity_line(a) + "\n" for a in xs)


def issue_json(i: Issue, comments: list[Comment], activity: list[Activity]) -> str:
    j = json_str
    return ('{"key":' + j(issue_key(i)) + ',"project":' + j(i.project_key) + ',"number":' + str(i.number)
            + ',"title":' + j(i.title) + ',"body":' + j(i.body) + ',"status":' + j(i.status)
            + ',"priority":' + str(i.priority) + ',"assignee":' + ("null" if i.assignee == "" else j(i.assignee))
            + ',"labels":[' + ",".join(j(l) for l in i.labels) + "]"
            + ',"comments":[' + ",".join('{"author":' + j(c.author) + ',"body":' + j(c.body) + "}" for c in comments) + "]"
            + ',"activity":[' + ",".join('{"kind":' + j(a.kind) + ',"detail":' + j(a.detail) + "}" for a in activity) + "]"
            + "}\n")


def issues_csv(xs: list[Issue]) -> str:
    rows = [",".join(csv_field(v) for v in [issue_key(i), i.title, i.status, str(i.priority), i.assignee,
                                             ";".join(i.labels), i.body]) + "\n" for i in xs]
    return "key,title,status,priority,assignee,labels,body\n" + "".join(rows)


# ---- tests (spec 5.3; underscore names so `from render import *` does not re-export them) ----

_ISS = Issue(id=1, project_id=1, project_key="WEB", number=7, title='Fix "login"', body="line1\nline2",
             status="open", priority=2, assignee="alice", labels=["bug", "ui"])
_WEB = Project(id=1, key="WEB", name="Web <site>")


class _TestRender(unittest.TestCase):
    def test_r_markdown(self):
        for src, want in [
            ("a **b** `c<d>` [x](http://e.com)\n\nnext",
             '<p>a <strong>b</strong> <code>c&lt;d&gt;</code> <a href="http://e.com">x</a></p>\n<p>next</p>'),
            ("[x](javascript:alert(1)) **open", "<p>[x](javascript:alert(1)) **open</p>"),
            ("<script>", "<p>&lt;script&gt;</p>"),
            ("line1\nline2", "<p>line1\nline2</p>"),
            ("  \n\na", "<p>a</p>"),
            ("", ""),
            ("**a [l](/x) b**", '<p><strong>a <a href="/x">l</a> b</strong></p>'),
        ]:
            self.assertEqual(md_render(src), want, src)

    def test_r_text_json(self):
        self.assertEqual(issue_line(_ISS), 'WEB-7 open P2 alice [bug,ui] Fix "login"\n')
        self.assertEqual(issue_line(replace(_ISS, assignee="", labels=[])), 'WEB-7 open P2 - [] Fix "login"\n')
        self.assertEqual(activity_text([Activity(1, 1, "created", ""), Activity(2, 1, "status", "closed")]),
                         "created\nstatus closed\n")
        self.assertEqual(comments_text([Comment(1, 1, "bob", "a\nb")]), "bob: a b\n")
        self.assertEqual(json_str('a"b\\c\nd\x01é'), '"a\\"b\\\\c\\nd\\u0001é"')
        self.assertEqual(
            issue_json(replace(_ISS, number=1, title='Say "hi"', assignee=""), [Comment(1, 1, "bob", "ok")],
                       [Activity(1, 1, "created", "")]),
            '{"key":"WEB-1","project":"WEB","number":1,"title":"Say \\"hi\\"","body":"line1\\nline2","status":"open",'
            '"priority":2,"assignee":null,"labels":["bug","ui"],"comments":[{"author":"bob","body":"ok"}],'
            '"activity":[{"kind":"created","detail":""}]}\n')  # spec example says priority 3, but iss has 2

    def test_r_csv(self):
        self.assertEqual(csv_field("plain"), "plain")
        self.assertEqual(csv_field("a,b"), '"a,b"')
        self.assertEqual(csv_field('say "hi"'), '"say ""hi"""')
        header = "key,title,status,priority,assignee,labels,body\n"
        self.assertEqual(issues_csv([replace(_ISS, number=1, title="a,b", body="x\ny", status="open", priority=3, assignee="")]),
                         header + 'WEB-1,"a,b",open,3,,bug;ui,"x\ny"\n')
        self.assertEqual(issues_csv([]), header)

    def test_r_query(self):
        self.assertEqual(url_encode("a b&c/é~"), "a%20b%26c%2F%C3%A9~")
        self.assertEqual(filter_query(Filter(status="open", assignee="", label="bug", priority=2, q="a b", page=1), 3),
                         "status=open&label=bug&priority=2&q=a%20b&page=3")
        self.assertEqual(filter_query(no_filter(), 2), "page=2")

    def test_r_pages(self):
        page = render_issue(replace(_ISS, title="<x>"), [], [])
        self.assertIn("<h1>WEB-7: &lt;x&gt;</h1>", page)
        self.assertNotIn("<x>", page)
        self.assertIn('action="/p/WEB/7/status"', page)
        self.assertIn("<button>close</button>", page)
        meta = render_issue_meta(replace(_ISS, assignee="", labels=[]))
        self.assertEqual(meta, '<p class="meta">Status: <span class="status">open</span> | Priority: P2 | Assignee: - | Labels: -</p>')
        self.assertIn("</h1>" + render_issue_meta(_ISS) + '<div class="body">', render_issue(_ISS, [], []))
        page = render_project(_WEB, [], no_filter(), IssuePage(issues=[_ISS], total=41, page=2, pages=3))
        for want in ["<h1>WEB: Web &lt;site&gt;</h1>", "Page 2 of 3 (41 issues)", 'href="/p/WEB?page=1">prev</a>',
                     'href="/p/WEB?page=3">next</a>', '<a href="/p/WEB/7">WEB-7</a>', 'action="/p/WEB/issues"']:
            self.assertIn(want, page)
        self.assertIn('<li><a href="/p/WEB">WEB</a> Web &lt;site&gt;</li>', render_home([_WEB]))
        page = html_page("a&b", "<i>x</i>")
        self.assertIn("<title>a&amp;b</title>", page)
        self.assertIn("<i>x</i></body></html>", page)
