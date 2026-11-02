# Tracker: specification

Tracker is an HTTP issue tracker over SQLite. It is built twice, in Overlang
(`examples/tracker/src/*.ov`) and in Python (`examples/tracker_py/*.py`), each
by four builders working in parallel who never talk to each other. Everything
two modules share (names, signatures, record fields, SQL, wire formats, error
messages) is fixed here. Where this document gives exact text, produce that
text byte for byte; where it gives behaviour, any implementation is fine.

Both ports must give byte-identical responses to the same request sequence,
so the smoke script of section 5.6 runs unchanged against either.

## 0. Conventions (read first)

- **Language.** Overlang code uses only `docs/SPEC.md` v0.1 features and the
  prelude. Python code is standard library only (3.10 or later; `match` is
  allowed).
- **One namespace.** The four `.ov` files form one flat namespace (no
  modules), so every top-level name in this spec is unique across all four
  files and none redefines a prelude name (`count`, `filter`, `lines`, `get`,
  `join`, `contains`, `replace`, `head`, `min`, `max`, ...). A builder may add
  private helpers, but their names must start with the module prefix: `db_`
  in `01_db.ov`, `m_` in `02_models.ov`, `r_` in `03_render.ov`, `rt_` in
  `04_routes.ov`. Test block names start with `t_db_`, `t_m_`, `t_r_`, `t_rt_`.
  In Python, private helpers start with `_` (so `from x import *` skips them).
- **Dependency order.** `01_db` < `02_models` < `03_render` < `04_routes`. A
  module may use only its own and earlier modules' names (they are added to
  the store in that order, and each `ov add` checks against what is stored).
- **No globals.** The database handle is threaded through every function
  that touches the database: Overlang `db: Int` (a handle >= 1; 0 or -1 is
  "no database"), Python `db: Db` where `Db = Optional[sqlite3.Connection]`.
- **Python mapping.** Same function names, same parameter names and order.
  `Int`/`Str`/`Bool` map to `int`/`str`/`bool`; `List[T]` to `list[T]`;
  record types to `@dataclass(frozen=True)` with the same fields in the same
  order; `Option[T]` to `Optional[T]` (`None`); `Result[T, Str]` to
  `tuple[Optional[T], str]`, `(value, "")` or `(None, message)`; tuples to
  tuples; `fn(A) -> R !fs` to `Callable[[A], R]`. Requires/ensures become
  `assert`s at the top/bottom of the function (the todo port's pattern).
  Python imports are star imports that mirror the flat namespace:
  `models.py` starts with `from db import *`, `render.py` with
  `from models import *`, `main.py` with `from render import *`.
- **Strings.** All lengths and indexes are in code points (Overlang `str_len`,
  `substr`, `chars`; Python `len`, slicing). "Trim" means prelude `trim` /
  Python `str.strip()` (ASCII whitespace; identical on the inputs we accept).
- **Ordering.** Issues by `number`, comments and activity by `id`, users by
  `id`, projects by `key`, labels by `name`. Always ascending.
- **No timestamps.** Nothing stores time, so every output is deterministic.
- **Line budgets (Overlang, tests included):** `01_db.ov` about 200 lines,
  `02_models.ov` about 550, `03_render.ov` about 650, `04_routes.ov` about 650:
  1,800 to 2,200 in total. The Python port may be shorter.
- **Working in parallel.** Until the earlier modules exist, check your module
  against a stub: copy the Overlang code blocks of section 3 for the earlier
  modules (they are valid Overlang: `type`, `foreign` and `declare fn`
  lines) into a scratch file and run `ov check stub.ov yourfile.ov`. Delete
  the stub when done; never commit it. Python builders can do the same with
  a scratch stub module. Final verification happens at assembly (section 4).

## 1. Schema

`schema_sql()` returns exactly this script (one string; whitespace inside
it is free). Foreign keys are documentation only (SQLite does not enforce
them by default and the app never turns them on).

```sql
create table if not exists users (
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
  unique (project_id, number)
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
```

Activity rows (kind, detail):

| Event | kind | detail |
|---|---|---|
| issue created | `created` | `""` |
| status changed (only when it actually changes) | `status` | the new status, `open` or `closed` |
| assignee changed (only when it actually changes) | `assigned` | the new assignee's name, or `-` when unassigned |
| comment added | `commented` | the author's name |

### 1.1 Exact SQL used by models

Every user value is bound as a `?` parameter (section 3.1); SQL text is only
ever built from the fixed fragments below. `?` are positional, left to right.

- **Issue select** (`issue_select_sql()`; 10 columns in `Issue` field order):

  ```sql
  select i.id, i.project_id, p.key, i.number, i.title, i.body, i.status, i.priority,
    coalesce(u.name, ''),
    coalesce((select group_concat(l.name, ',' order by l.name) from issue_labels il
      join labels l on l.id = il.label_id where il.issue_id = i.id), '')
  from issues i join projects p on p.id = i.project_id left join users u on u.id = i.assignee_id
  ```

- **Filter fragments** (`issue_where(f)`), appended in this order, each only
  when the filter field is set, with its parameters in the same order:

  | Field set when | Fragment | Params |
  |---|---|---|
  | `f.status != ""` | ` and i.status = ?` | `p_str(f.status)` |
  | `f.assignee != ""` | ` and u.name = ?` | `p_str(f.assignee)` |
  | `f.label != ""` | ` and exists (select 1 from issue_labels il2 join labels l2 on l2.id = il2.label_id where il2.issue_id = i.id and l2.name = ?)` | `p_str(f.label)` |
  | `f.priority > 0` | ` and i.priority = ?` | `p_int(f.priority)` |
  | `f.q != ""` | ` and (i.title like ? escape '\' or i.body like ? escape '\')` | `p_str(like_pattern(f.q))` twice |

  (The SQL text contains one backslash between the quotes: `escape '\'`. In
  an Overlang or Python literal that is `'\\'`.)

- **Page of issues:** `issue_select_sql() ++ " where i.project_id = ?" ++ where ++ " order by i.number limit 20 offset ?"`,
  params `[p_int(project_id)] ++ where_params ++ [p_int(page_offset(f.page))]`.
- **Count:** `"select count(*) from issues i left join users u on u.id = i.assignee_id where i.project_id = ?" ++ where`,
  params `[p_int(project_id)] ++ where_params`.
- **All issues of a project (CSV):** `issue_select_sql() ++ " where i.project_id = ? order by i.number"`.
- **One issue:** `issue_select_sql() ++ " where i.project_id = ? and i.number = ?"`.
- **Create issue** (one statement, so the per-project number is atomic), via
  `db_int`, returning the new id:

  ```sql
  insert into issues (project_id, number, title, body, priority, assignee_id)
  select ?, coalesce(max(number), 0) + 1, ?, ?, ?, ? from issues where project_id = ?
  returning id
  ```

  params `[p_int(project_id), p_str(title), p_str(body), p_int(priority), assignee, p_int(project_id)]`
  where `assignee` is `p_null()` when `assignee_id == 0`, else `p_int(assignee_id)`.
  Then `select number from issues where id = ?` gives the number to return.
- Other inserts use `insert into T (...) values (...) returning id` with
  `db_int`. `add_label` uses `insert or ignore into issue_labels (issue_id, label_id) values (?, ?)`.
- **Comments:** `select c.id, c.issue_id, u.name, c.body from comments c join users u on u.id = c.author_id where c.issue_id = ? order by c.id`.
- **Activity:** `select id, issue_id, kind, detail from activity where issue_id = ? order by id`.

## 2. Routes

Run: Overlang `./tracker [db_path] [port]`, Python `python3 main.py [db_path] [port]`.
Defaults `tracker.db` and `8080`; a port that does not parse as an Int falls
back to 8080. The server binds 127.0.0.1, serves one request at a time, and
prints `listening on http://127.0.0.1:<port>` (then flushes) once it is ready.
If the database cannot be opened it prints `cannot open <db_path>` and exits
1; if it cannot listen, `cannot listen on port <port>` and exits 1.

### 2.1 Responses

| Kind | Status | Content-Type | Body |
|---|---|---|---|
| HTML | 200 | `text/html; charset=utf-8` | a page from section 2.4 |
| text | 200 | `text/plain; charset=utf-8` | lines of section 2.3, each ending `\n`; empty list is the empty body |
| JSON | 200 | `application/json` | section 2.3 |
| CSV | 200 | `text/csv; charset=utf-8` | section 2.3 |
| error | 400, 404, 422 | `text/plain; charset=utf-8` | the message of the table below, then `\n` |

POST handlers answer with a page (200), never a redirect: the HTTP shim cannot
set headers. Any method other than the one listed for a path, and any path not
listed, is 404 `not found`.

Status meanings: **404** the route, project or issue does not exist. **400** a
query or form value is malformed (not a number where one is required, an
unknown filter value). **422** the request is well formed but invalid (blank
required field, bad name format, duplicate, unknown user or label, out-of-range
priority on a write).

### 2.2 Route table

`{KEY}` is a path segment used verbatim (not decoded) as a project key; `{N}`
is a segment of ASCII digits with value >= 1 (`issue_ref`); a segment that is
not, such as `/p/WEB/abc` or `/p/WEB/0`, falls through to 404 `not found`.
Paths are split on `/` with empty segments dropped, so `/p/WEB/` is
`/p/WEB`. The query string is ignored on routes that do not list it.

Form fields (POST bodies, `application/x-www-form-urlencoded`) and query
values are read with `form_value` and then cleaned: a **line** field
(`form_line`) has `\r` and `\n` turned into spaces, `\x1e` and `\x1f` into
spaces, and is trimmed; a **text** field (`form_text`) has `\r\n` and lone `\r`
turned into `\n`, `\x1e` and `\x1f` into spaces, and is trimmed. Only `body`
fields are text fields; all others are line fields. An absent field reads as `""`.

For every route under `/p/{KEY}`, an unknown project is checked first: 404
`no such project: {KEY}`. For routes under `/p/{KEY}/{N}`, an unknown issue is
next: 404 `no such issue: {KEY}-{N}`. Then the checks listed, in the order
listed; the first failure is the response.

| # | Method | Path | Inputs | Checks, in order (status `message`) | Success response |
|---|---|---|---|---|---|
| 1 | GET | `/` | | | HTML `render_home(list_projects)` |
| 2 | GET | `/health` | | | text `ok\n` |
| 3 | GET | `/users` | | | HTML `render_users(list_users)` |
| 4 | GET | `/users.txt` | | | text `users_text` |
| 5 | POST | `/users` | `name` | empty: 422 `name required`; `!valid_user_name`: 422 `bad name`; exists: 422 `user exists` | create; HTML `render_users` |
| 6 | GET | `/projects.txt` | | | text `projects_text` |
| 7 | POST | `/projects` | `key`, `name` | key empty: 422 `key required`; `!valid_project_key`: 422 `bad key`; name empty: 422 `name required`; key exists: 422 `project exists` | create; HTML project page, `no_filter()` |
| 8 | GET | `/p/{KEY}` | query `status assignee label priority q page` | `parse_filter` error: 400 with its message | HTML project page |
| 9 | GET | `/p/{KEY}/issues.txt` | same query | same | text `issues_text` of the page's issues |
| 10 | GET | `/p/{KEY}/issues.csv` | | | CSV `issues_csv(all_issues)` |
| 11 | POST | `/p/{KEY}/issues` | `title body priority assignee labels` | title empty: 422 `title required`; priority (empty means 3) not `parse_nat`-able: 400 `bad priority`; outside 1..4: 422 `bad priority`; assignee non-empty and unknown: 422 `no such user: {assignee}`; first unknown name in `parse_label_names(labels)`: 422 `no such label: {name}` | create; HTML issue page |
| 12 | GET | `/p/{KEY}/labels.txt` | | | text `labels_text` |
| 13 | POST | `/p/{KEY}/labels` | `name` | empty: 422 `name required`; `!valid_label_name`: 422 `bad label`; exists in project: 422 `label exists` | create; HTML project page, `no_filter()` |
| 14 | GET | `/p/{KEY}/{N}` | | | HTML issue page |
| 15 | GET | `/p/{KEY}/{N}.json` | | | JSON `issue_json` |
| 16 | GET | `/p/{KEY}/{N}.txt` | | | text `issue_line` |
| 17 | GET | `/p/{KEY}/{N}/comments.txt` | | | text `comments_text` |
| 18 | GET | `/p/{KEY}/{N}/activity.txt` | | | text `activity_text` |
| 19 | POST | `/p/{KEY}/{N}/edit` | `title body priority` | priority non-empty and not `parse_nat`-able: 400 `bad priority`; outside 1..4: 422 `bad priority` | each empty field keeps its current value; `edit_issue`; HTML issue page |
| 20 | POST | `/p/{KEY}/{N}/status` | `status` | `!valid_status`: 422 `bad status` | `set_status`; HTML issue page |
| 21 | POST | `/p/{KEY}/{N}/assign` | `assignee` | non-empty and unknown: 422 `no such user: {assignee}` | empty unassigns; `set_assignee`; HTML issue page |
| 22 | POST | `/p/{KEY}/{N}/comments` | `author body` | author empty: 422 `author required`; unknown: 422 `no such user: {author}`; body empty: 422 `body required` | `add_comment`; HTML issue page |
| 23 | POST | `/p/{KEY}/{N}/labels` | `label` | empty: 422 `label required`; unknown in project: 422 `no such label: {label}` | `add_label` (no-op if present); HTML issue page |
| 24 | POST | `/p/{KEY}/{N}/labels/remove` | `label` | same as 23 | `remove_label` (no-op if absent); HTML issue page |
| 25 | any | anything else | | 404 `not found` | |

The "HTML project page" is `render_project(p, list_labels, f, search_issues(f))`;
the "HTML issue page" is `render_issue(i, list_comments, list_activity)` for the
issue re-read from the database after any change. Route 9 uses the same
`search_issues` call, so it is paginated too.

`parse_filter(query)` reads each key with `form_line`, in this order:
`status` must be `""`, `open` or `closed`, else `bad status`; `priority` is
`""` (0, no filter) or `parse_nat` in 1..4, else `bad priority`; `page` is
`""` (1) or `parse_nat` in 1..1000000, else `bad page`; `assignee`, `label`
and `q` are taken as they are (`""` means no filter). An `assignee` or
`label` that matches nothing simply yields no issues.

### 2.3 Text, JSON and CSV formats

In these formats values are written raw (no HTML escaping). `{key}` below is
`issue_key(i)`, for example `WEB-12`.

- `issue_line(i)`: `{key} {status} P{priority} {assignee or -} [{labels joined by ","}] {title}\n`,
  for example `WEB-7 open P2 alice [bug,ui] Fix "login"\n` or `WEB-8 closed P3 - [] Typo\n`.
- `users_text`: `{id} {name}\n` per user. `projects_text`: `{key} {name}\n`.
  `labels_text`: `{name}\n`.
- `comments_text`: `{author}: {body with each "\n" replaced by " "}\n` per comment.
- `activity_line(a)`: `{kind}` when `detail` is empty, else `{kind} {detail}`;
  `activity_text` adds `\n` after each: `created\nstatus closed\nassigned alice\ncommented bob\n`.
- `issue_json(i, comments, activity)`: compact JSON, keys in this order, then `\n`:

  ```
  {"key":"WEB-1","project":"WEB","number":1,"title":"Say \"hi\"","body":"line1\nline2","status":"open","priority":3,"assignee":null,"labels":["bug","ui"],"comments":[{"author":"bob","body":"ok"}],"activity":[{"kind":"created","detail":""}]}
  ```

  `assignee` is `null` when empty, else a string. `json_str(s)` is `"` + s
  with `\` as `\\`, `"` as `\"`, newline as `\n`, carriage return as `\r`, tab
  as `\t`, any other code point below 0x20 as `\u00XX` (lowercase hex, 4
  digits), everything else (including `/` and non-ASCII) unchanged, + `"`.
  Do not use Python's `json.dumps`; it differs (`\b`, `\f`, `ensure_ascii`).
- `issues_csv(xs)`: the header line `key,title,status,priority,assignee,labels,body\n`,
  then per issue the seven fields `{key}`, title, status, priority, assignee
  (empty when none), labels joined by `;`, body, each through `csv_field`,
  joined by `,`, ending `\n`. `csv_field(s)` is `s` unchanged unless it
  contains `,` `"` `\n` or `\r`; then `"` + s with each `"` doubled + `"`.
  Example: `WEB-1,"a,b",open,3,,bug;ui,"x\ny"\n` (the `\n` inside the quotes is
  a real newline).

### 2.4 HTML pages

Notation: in the templates below `{x}` means `html_escape(x)`, `{!x}` means
`x` inserted raw (it is already HTML), and everything else is literal text.
There are no newlines in any template. `html_escape` is the shim's: `&` `<` `>`
`"` `'` become `&amp;` `&lt;` `&gt;` `&quot;` `&#39;`.

- `html_page(title, body)`:
  `<!doctype html><html><head><meta charset="utf-8"><title>{title}</title></head><body><nav><a href="/">Tracker</a> <a href="/users">Users</a></nav>{!body}</body></html>`
- `render_home(projects)`: `html_page("Tracker", ...)` with body
  `<h1>Projects</h1><ul class="projects">` + per project
  `<li><a href="/p/{key}">{key}</a> {name}</li>` +
  `</ul><form method="post" action="/projects"><input name="key" placeholder="KEY"> <input name="name" placeholder="name"> <button>create project</button></form>`
- `render_users(users)`: `html_page("Users", ...)` with body
  `<h1>Users</h1><ul class="users">` + per user `<li>{name}</li>` +
  `</ul><form method="post" action="/users"><input name="name" placeholder="name"> <button>create user</button></form>`
- `render_issue_row(i)`:
  `<tr><td><a href="/p/{project_key}/{number}">{issue_key}</a></td><td>{title}</td><td>{status}</td><td>P{priority}</td><td>{dash_if_empty(assignee)}</td><td>{labels_display(labels)}</td></tr>`
- `render_pager(p, f, pg)`: `<p class="pager">Page {page} of {pages} ({total} issues)`
  + if `pg.page > 1`: ` <a href="{href(pg.page - 1)}">prev</a>`
  + if `pg.page < pg.pages`: ` <a href="{href(pg.page + 1)}">next</a>`
  + `</p>`, where `href(n)` is `"/p/" ++ p.key ++ "?" ++ filter_query(f, n)`
  (escaped like every `{}`, so `&` appears as `&amp;`).
- `render_project(p, labels, f, pg)`: `html_page(p.key ++ ": " ++ p.name, ...)` with body
  `<h1>{key}: {name}</h1>`
  + `<form method="get" action="/p/{key}"><input name="status" value="{f.status}" placeholder="status"> <input name="assignee" value="{f.assignee}" placeholder="assignee"> <input name="label" value="{f.label}" placeholder="label"> <input name="priority" value="{P}" placeholder="priority"> <input name="q" value="{f.q}" placeholder="search"> <button>filter</button></form>`
  (`P` is `show(f.priority)` when > 0, else empty)
  + `<table class="issues"><tr><th>Issue</th><th>Title</th><th>Status</th><th>Priority</th><th>Assignee</th><th>Labels</th></tr>`
  + `render_issue_row` per issue of `pg` + `</table>`
  + `{!render_pager(p, f, pg)}`
  + `<h2>Labels</h2><p class="labels">{labels_display(label names)}</p>`
  + `<form method="post" action="/p/{key}/labels"><input name="name" placeholder="label"> <button>add label</button></form>`
  + `<h2>New issue</h2><form method="post" action="/p/{key}/issues"><input name="title" placeholder="title"> <input name="priority" value="3"> <input name="assignee" placeholder="assignee"> <input name="labels" placeholder="labels"> <textarea name="body"></textarea> <button>create issue</button></form>`
- `render_issue(i, comments, activity)`: `html_page(issue_key(i) ++ ": " ++ i.title, ...)`;
  with `B` = `/p/{project_key}/{number}`, body
  `<h1>{issue_key}: {title}</h1>`
  + `<p class="meta">Status: <span class="status">{status}</span> | Priority: P{priority} | Assignee: {dash_if_empty(assignee)} | Labels: {labels_display(labels)}</p>`
  + `<div class="body">{!md_render(body)}</div>`
  + `<form method="post" action="B/status"><input type="hidden" name="status" value="{T}"><button>{V}</button></form>`
  (`T`/`V` are `closed`/`close` when the issue is open, `open`/`reopen` when closed)
  + `<form method="post" action="B/assign"><input name="assignee" value="{assignee}"> <button>assign</button></form>`
  + `<form method="post" action="B/labels"><input name="label" placeholder="label"> <button>add label</button></form>`
  + `<form method="post" action="B/labels/remove"><input name="label" placeholder="label"> <button>remove label</button></form>`
  + `<form method="post" action="B/edit"><input name="title" value="{title}"> <input name="priority" value="{priority}"> <textarea name="body">{body}</textarea> <button>save</button></form>`
  + `<h2>Comments</h2><ul class="comments">` + per comment `<li><b>{author}</b> {!md_render(body)}</li>` + `</ul>`
  + `<form method="post" action="B/comments"><input name="author" placeholder="author"> <textarea name="body"></textarea> <button>comment</button></form>`
  + `<h2>Activity</h2><ul class="activity">` + per entry `<li>{activity_line(a)}</li>` + `</ul>`
- `dash_if_empty(s)` is `-` for `""`, else `s`. `labels_display(xs)` is the
  names joined by `, `, or `-` for an empty list.

### 2.5 Markdown-lite (`md_render`, `md_inline`)

`md_render(s)`: split `s` into lines on `\n` (prelude `lines`; Python
`s.split("\n")`, dropping one trailing empty string). A line that is empty
after trimming is blank. Each maximal run of non-blank lines is a paragraph:
its lines, untrimmed, joined with `\n`. The result is the paragraphs, each as
`<p>` + `md_inline(paragraph)` + `</p>`, joined with `\n`. No paragraphs gives
`""`.

`md_inline(s)`: first `t = html_escape(s)`. Then scan `t` with a position
`k` from 0, appending to the output; at each `k`, try in this order:

1. `` ` `` at `k` and another `` ` `` at the first later position `e`:
   append `<code>` + `t[k+1..e]` + `</code>` (no further processing), `k = e + 1`.
2. `**` at `k` and another `**` at the first position `e >= k + 3` (so the
   content is non-empty): append `<strong>` + `r_inline_rest(t[k+2..e])` +
   `</strong>`, `k = e + 2`, where the content is processed by these same
   rules (it is already escaped: do not escape twice).
3. `[` at `k`; `j` the first position after `k` where `](` starts; `e` the
   first `)` after `j + 2`; and the URL `t[j+2..e]` starts with `http://`,
   `https://` or `/`: append `<a href="` + URL + `">` + `t[k+1..j]` + `</a>`
   (text not processed further), `k = e + 1`.
4. Otherwise append the one code point at `k`, `k = k + 1`.

`t[a..b]` is the half-open code-point range. Since escaping comes first, `"`
in a URL is already `&quot;` and cannot end the attribute; a `javascript:`
URL fails rule 3 and stays literal text. Implement the processing of
already-escaped text as a private helper (`r_inline_rest` in Overlang,
`_inline_rest` in Python); `md_inline(s)` is `r_inline_rest(html_escape(s))`.

Examples (the expected values of the render tests), as `md_render(input) == output`
in Overlang literal syntax:

```
md_render("a **b** `c<d>` [x](http://e.com)\n\nnext") == "<p>a <strong>b</strong> <code>c&lt;d&gt;</code> <a href=\"http://e.com\">x</a></p>\n<p>next</p>"
md_render("[x](javascript:alert(1)) **open") == "<p>[x](javascript:alert(1)) **open</p>"
md_render("<script>") == "<p>&lt;script&gt;</p>"
md_render("line1\nline2") == "<p>line1\nline2</p>"
md_render("  \n\na") == "<p>a</p>"
md_render("") == ""
md_render("**a [l](/x) b**") == "<p><strong>a <a href=\"/x\">l</a> b</strong></p>"
```

### 2.6 Query strings

`url_encode(s)`: every code point in `A-Z a-z 0-9 - _ . ~` is kept; every
other one is written as `%XX` per byte of its UTF-8 encoding, uppercase hex
(space is `%20`). Python: `urllib.parse.quote(s, safe="")`.

`filter_query(f, page)`: the pairs `status`, `assignee`, `label`, `priority`,
`q` in that order, each only when set (`!= ""`, priority `> 0`), as
`name=url_encode(value)`, then always `page={page}`, joined by `&`. Example:
`status=open&label=bug&priority=2&q=a%20b&page=3`; with no filter, `page=2`.

## 3. Modules

Signatures below are exact. Each Overlang block is valid Overlang (checked
with `ov check`); turn each `declare fn` into `fn ... { body }` with the same
signature, contracts included. The comment above each line is its behaviour.

### 3.1 `01_db` (Overlang `src/01_db.ov`, Python `db.py`)

**Shims.** Of `runtime/ext/ov_sqlite.h` the db module binds `ov_sqlite_open`,
`ov_sqlite_close`, `ov_sqlite_error`, `ov_sqlite_exec` (the schema script),
and two new functions that the Overlang db builder adds to
`runtime/ext/ov_sqlite.h` and `runtime/ext/ov_sqlite.c` (no other runtime file;
never `src/`). The `_1s/_2s/_1i` variants are not used: every statement here
needs more parameters than they take.

```c
/* Like ov_sqlite_exec, but runs only the first statement of sql, with its ? parameters bound from params. */
int64_t ov_sqlite_exec_p(int64_t db, const char* sql, const char* params);
/* Like ov_sqlite_query, but runs only the first statement of sql, with its ? parameters bound from params. */
const char* ov_sqlite_query_p(int64_t db, const char* sql, const char* params);
```

Parameter encoding: `params` is `""` for no parameters, otherwise fields
separated by `\x1f`; field `k` (1-based) binds `?k`. A field's first byte is
its tag: `i` then a decimal int64 (optional `-`, then one or more digits,
nothing else) binds an integer; `s` then any text (possibly empty) binds text;
`n` alone binds NULL. An empty field, an unknown tag, a malformed or
out-of-range integer, or more than 32 fields: return -1 (exec) or NULL (query)
and set the error to `bad parameter`. A field beyond the statement's parameter
count is SQLite's own range error (also -1/NULL); unbound `?` are NULL. Both run
only the first statement even when `params` is `""` (unlike `ov_sqlite_exec`).
Implementation hint: give `param` a `null` flag, and give `run` an explicit
"first statement only" flag instead of deriving it from `np`.
Add CHECKs for these to `runtime/ext/tests/test_sqlite.c` and run `make test`
in `runtime/ext`. The compiler embeds the shims (`include_str!` in
`src/build.rs`), so after editing them run `cargo build --release` in the repo
root. Only the Overlang db builder edits `runtime/ext` or rebuilds `ov`.

The Python `db.py` mirrors the shims with the same names and the same
`params` encoding (so the encoding is tested in both ports): it keeps a
module-level last-error string like the todo port, opens with
`sqlite3.connect(path, isolation_level=None)`, and decodes `params` into a
tuple (`int(...)`, `str`, `None`), raising nothing (errors give -1/None).

```overlang
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_open(path: Str) -> Int !fs = "ov_sqlite_open"
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_close(db: Int) -> Unit !fs = "ov_sqlite_close"
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_error(db: Int) -> Str !fs = "ov_sqlite_error"
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec(db: Int, sql: Str) -> Int !fs = "ov_sqlite_exec"
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec_p(db: Int, sql: Str, params: Str) -> Int !fs = "ov_sqlite_exec_p"
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_query_p(db: Int, sql: Str, params: Str) -> Option[Str] !fs = "ov_sqlite_query_p"

// The DDL of section 1 as one script.
declare fn schema_sql() -> Str
// sqlite_open(path), then sqlite_exec(schema_sql()); the handle, or -1 if either fails (closing the handle).
declare fn db_open(path: Str) -> Int !fs
// "i" ++ show(n).
declare fn p_int(n: Int) -> Str
// "s" ++ s. The caller guarantees s has no \x1f or \x1e (routes clean every input).
declare fn p_str(s: Str) -> Str
// "n".
declare fn p_null() -> Str
// The encoded params joined by "\u{1f}"; [] gives "".
declare fn sql_params(ps: List[Str]) -> Str
// sqlite_exec_p(db, sql, sql_params(ps)): rows changed, or -1.
declare fn db_exec(db: Int, sql: Str, ps: List[Str]) -> Int !fs
// sqlite_query_p then parse_table; [] on error.
declare fn db_rows(db: Int, sql: Str, ps: List[Str]) -> List[List[Str]] !fs
// The first column of the first row of db_rows as an Int; -1 if there is no row or it is not an Int.
declare fn db_int(db: Int, sql: Str, ps: List[Str]) -> Int !fs
// Shim text to rows: split on "\u{1e}", drop the header row, split each row on "\u{1f}"; "" gives [].
declare fn parse_table(text: Str) -> List[List[Str]]
// The i-th cell, or "" when out of range.
declare fn cell_str(row: List[Str], i: Int) -> Str
// The i-th cell parsed with parse_int, or 0.
declare fn cell_int(row: List[Str], i: Int) -> Int
```

```python
Db = Optional[sqlite3.Connection]
def sqlite_open(path: str) -> Db: ...
def sqlite_close(db: Db) -> None: ...
def sqlite_error(db: Db) -> str: ...
def sqlite_exec(db: Db, sql: str) -> int: ...
def sqlite_exec_p(db: Db, sql: str, params: str) -> int: ...
def sqlite_query_p(db: Db, sql: str, params: str) -> Optional[str]: ...
def schema_sql() -> str: ...
def db_open(path: str) -> Db: ...                       # None on failure
def p_int(n: int) -> str: ...
def p_str(s: str) -> str: ...
def p_null() -> str: ...
def sql_params(ps: list[str]) -> str: ...
def db_exec(db: Db, sql: str, ps: list[str]) -> int: ...
def db_rows(db: Db, sql: str, ps: list[str]) -> list[list[str]]: ...
def db_int(db: Db, sql: str, ps: list[str]) -> int: ...
def parse_table(text: str) -> list[list[str]]: ...
def cell_str(row: list[str], i: int) -> str: ...
def cell_int(row: list[str], i: int) -> int: ...
```

Python `sqlite_query_p` renders values like the shim: `None` as `""`,
integers in decimal, text as is.

### 3.2 `02_models` (Overlang `src/02_models.ov`, Python `models.py`)

```overlang
type User = { id: Int, name: Str }
type Project = { id: Int, key: Str, name: Str }
type Label = { id: Int, project_id: Int, name: Str }
// assignee is the user's name, "" when unassigned; labels are names sorted ascending.
type Issue = { id: Int, project_id: Int, project_key: Str, number: Int, title: Str, body: Str, status: Str, priority: Int, assignee: Str, labels: List[Str] }
// author is the user's name.
type Comment = { id: Int, issue_id: Int, author: Str, body: Str }
type Activity = { id: Int, issue_id: Int, kind: Str, detail: Str }
// "" and 0 mean "no filter"; page is 1-based.
type Filter = { status: Str, assignee: Str, label: Str, priority: Int, q: Str, page: Int }
type IssuePage = { issues: List[Issue], total: Int, page: Int, pages: Int }

// 1 to 32 code points, each an ASCII letter, digit, "_", "-" or ".".
declare fn valid_user_name(s: Str) -> Bool
// 2 to 10 code points: an ASCII uppercase letter, then uppercase letters or digits.
declare fn valid_project_key(s: Str) -> Bool
// 1 to 20 code points, each an ASCII lowercase letter, digit or "-".
declare fn valid_label_name(s: Str) -> Bool
// s == "open" || s == "closed".
declare fn valid_status(s: Str) -> Bool
// i.project_key ++ "-" ++ show(i.number).
declare fn issue_key(i: Issue) -> Str
// "%" ++ q with "\" -> "\\", "%" -> "\%", "_" -> "\_" (backslash first) ++ "%".
declare fn like_pattern(q: Str) -> Str
// max(1, (total + 19) / 20).
declare fn page_count(total: Int) -> Int
  requires total >= 0 && total <= 1_000_000_000
  ensures result >= 1
// (page - 1) * 20.
declare fn page_offset(page: Int) -> Int
  requires page >= 1 && page <= 1_000_000
  ensures result >= 0
// Filter { status: "", assignee: "", label: "", priority: 0, q: "", page: 1 }.
declare fn no_filter() -> Filter
// Row parsers over db_rows output, in the column order of section 1.1 (cell_str/cell_int).
declare fn row_user(r: List[Str]) -> User
declare fn row_project(r: List[Str]) -> Project
declare fn row_label(r: List[Str]) -> Label
// Column 9 split on ",", empty pieces dropped.
declare fn row_issue(r: List[Str]) -> Issue
declare fn row_comment(r: List[Str]) -> Comment
declare fn row_activity(r: List[Str]) -> Activity
// The issue select of section 1.1.
declare fn issue_select_sql() -> Str
// The filter fragments and their params, section 1.1 (page is not included).
declare fn issue_where(f: Filter) -> (Str, List[Str])
// Insert, returning the new id.
declare fn create_user(db: Int, name: Str) -> Int !fs
  requires valid_user_name(name)
// select id, name from users order by id.
declare fn list_users(db: Int) -> List[User] !fs
declare fn find_user(db: Int, name: Str) -> Option[User] !fs
declare fn create_project(db: Int, key: Str, name: Str) -> Int !fs
  requires valid_project_key(key) && str_len(trim(name)) > 0
// select id, key, name from projects order by key.
declare fn list_projects(db: Int) -> List[Project] !fs
declare fn find_project(db: Int, key: Str) -> Option[Project] !fs
declare fn create_label(db: Int, project_id: Int, name: Str) -> Int !fs
  requires valid_label_name(name)
// select id, project_id, name from labels where project_id = ? order by name.
declare fn list_labels(db: Int, project_id: Int) -> List[Label] !fs
declare fn find_label(db: Int, project_id: Int, name: Str) -> Option[Label] !fs
// Insert (section 1.1; assignee_id 0 = none), attach each label id, log "created"; returns the new issue's number.
declare fn create_issue(db: Int, project_id: Int, title: Str, body: Str, priority: Int, assignee_id: Int, label_ids: List[Int]) -> Int !fs
  requires str_len(trim(title)) > 0 && priority >= 1 && priority <= 4
declare fn find_issue(db: Int, project_id: Int, number: Int) -> Option[Issue] !fs
// Number of issues matching f (ignoring f.page).
declare fn count_issues(db: Int, project_id: Int, f: Filter) -> Int !fs
// One page (20) of matching issues; total = count_issues, pages = page_count(total), page = f.page.
declare fn search_issues(db: Int, project_id: Int, f: Filter) -> IssuePage !fs
  requires f.page >= 1 && f.page <= 1_000_000
// Every issue of the project, by number.
declare fn all_issues(db: Int, project_id: Int) -> List[Issue] !fs
// update issues set title = ?, body = ?, priority = ? where id = ?; rows changed. No activity.
declare fn edit_issue(db: Int, i: Issue, title: Str, body: Str, priority: Int) -> Int !fs
  requires priority >= 1 && priority <= 4
// If status differs: update and log ("status", status), true; else false.
declare fn set_status(db: Int, i: Issue, status: Str) -> Bool !fs
  requires valid_status(status)
// If the new name (user's name or "") differs from i.assignee: update assignee_id (NULL for None) and log ("assigned", name or "-"), true; else false.
declare fn set_assignee(db: Int, i: Issue, user: Option[User]) -> Bool !fs
// insert or ignore into issue_labels; rows changed.
declare fn add_label(db: Int, i: Issue, l: Label) -> Int !fs
// delete from issue_labels where issue_id = ? and label_id = ?; rows changed.
declare fn remove_label(db: Int, i: Issue, l: Label) -> Int !fs
// Insert the comment, log ("commented", author.name); returns the comment id.
declare fn add_comment(db: Int, i: Issue, author: User, body: Str) -> Int !fs
declare fn list_comments(db: Int, issue_id: Int) -> List[Comment] !fs
// Insert one activity row; returns its id.
declare fn log_activity(db: Int, issue_id: Int, kind: Str, detail: Str) -> Int !fs
declare fn list_activity(db: Int, issue_id: Int) -> List[Activity] !fs
```

```python
@dataclass(frozen=True)
class User: id: int; name: str                                     # one field per line in the file
class Project: id: int; key: str; name: str
class Label: id: int; project_id: int; name: str
class Issue: id: int; project_id: int; project_key: str; number: int; title: str; body: str; status: str; priority: int; assignee: str; labels: list[str]
class Comment: id: int; issue_id: int; author: str; body: str
class Activity: id: int; issue_id: int; kind: str; detail: str
class Filter: status: str; assignee: str; label: str; priority: int; q: str; page: int
class IssuePage: issues: list[Issue]; total: int; page: int; pages: int
def valid_user_name(s: str) -> bool: ...
def valid_project_key(s: str) -> bool: ...
def valid_label_name(s: str) -> bool: ...
def valid_status(s: str) -> bool: ...
def issue_key(i: Issue) -> str: ...
def like_pattern(q: str) -> str: ...
def page_count(total: int) -> int: ...
def page_offset(page: int) -> int: ...
def no_filter() -> Filter: ...
def row_user(r: list[str]) -> User: ...
def row_project(r: list[str]) -> Project: ...
def row_label(r: list[str]) -> Label: ...
def row_issue(r: list[str]) -> Issue: ...
def row_comment(r: list[str]) -> Comment: ...
def row_activity(r: list[str]) -> Activity: ...
def issue_select_sql() -> str: ...
def issue_where(f: Filter) -> tuple[str, list[str]]: ...
def create_user(db: Db, name: str) -> int: ...
def list_users(db: Db) -> list[User]: ...
def find_user(db: Db, name: str) -> Optional[User]: ...
def create_project(db: Db, key: str, name: str) -> int: ...
def list_projects(db: Db) -> list[Project]: ...
def find_project(db: Db, key: str) -> Optional[Project]: ...
def create_label(db: Db, project_id: int, name: str) -> int: ...
def list_labels(db: Db, project_id: int) -> list[Label]: ...
def find_label(db: Db, project_id: int, name: str) -> Optional[Label]: ...
def create_issue(db: Db, project_id: int, title: str, body: str, priority: int, assignee_id: int, label_ids: list[int]) -> int: ...
def find_issue(db: Db, project_id: int, number: int) -> Optional[Issue]: ...
def count_issues(db: Db, project_id: int, f: Filter) -> int: ...
def search_issues(db: Db, project_id: int, f: Filter) -> IssuePage: ...
def all_issues(db: Db, project_id: int) -> list[Issue]: ...
def edit_issue(db: Db, i: Issue, title: str, body: str, priority: int) -> int: ...
def set_status(db: Db, i: Issue, status: str) -> bool: ...
def set_assignee(db: Db, i: Issue, user: Optional[User]) -> bool: ...
def add_label(db: Db, i: Issue, l: Label) -> int: ...
def remove_label(db: Db, i: Issue, l: Label) -> int: ...
def add_comment(db: Db, i: Issue, author: User, body: str) -> int: ...
def list_comments(db: Db, issue_id: int) -> list[Comment]: ...
def log_activity(db: Db, issue_id: int, kind: str, detail: str) -> int: ...
def list_activity(db: Db, issue_id: int) -> list[Activity]: ...
```

`Issue.labels` is a list, so the Python `Issue` dataclass is frozen but not
hashable; nothing hashes it.

### 3.3 `03_render` (Overlang `src/03_render.ov`, Python `render.py`)

Every function here is pure. Formats are in sections 2.3 to 2.6.

```overlang
foreign c "ov_http.h" fn html_escape(s: Str) -> Str = "ov_http_html_escape"

// Section 2.6.
declare fn url_encode(s: Str) -> Str
// Section 2.3, quotes included.
declare fn json_str(s: Str) -> Str
// Section 2.3.
declare fn csv_field(s: Str) -> Str
// Section 2.5: escape, then inline rules.
declare fn md_inline(s: Str) -> Str
// Section 2.5: paragraphs.
declare fn md_render(s: Str) -> Str
// Section 2.4 page skeleton.
declare fn html_page(title: Str, body: Str) -> Str
// "-" for "", else s.
declare fn dash_if_empty(s: Str) -> Str
// Names joined by ", ", or "-" when empty.
declare fn labels_display(names: List[Str]) -> Str
// Section 2.6.
declare fn filter_query(f: Filter, page: Int) -> Str
// Section 2.4 pages and fragments.
declare fn render_home(projects: List[Project]) -> Str
declare fn render_users(users: List[User]) -> Str
declare fn render_issue_row(i: Issue) -> Str
declare fn render_pager(p: Project, f: Filter, pg: IssuePage) -> Str
declare fn render_project(p: Project, labels: List[Label], f: Filter, pg: IssuePage) -> Str
declare fn render_issue(i: Issue, comments: List[Comment], activity: List[Activity]) -> Str
// Section 2.3 text formats.
declare fn issue_line(i: Issue) -> Str
declare fn issues_text(xs: List[Issue]) -> Str
declare fn users_text(xs: List[User]) -> Str
declare fn projects_text(xs: List[Project]) -> Str
declare fn labels_text(xs: List[Label]) -> Str
declare fn comments_text(xs: List[Comment]) -> Str
declare fn activity_line(a: Activity) -> Str
declare fn activity_text(xs: List[Activity]) -> Str
// Section 2.3 JSON, ending "\n".
declare fn issue_json(i: Issue, comments: List[Comment], activity: List[Activity]) -> Str
// Section 2.3 CSV, header line included.
declare fn issues_csv(xs: List[Issue]) -> Str
```

```python
def html_escape(s: str) -> str: ...      # the todo port's version: & < > " ' as &amp; &lt; &gt; &quot; &#39;
def url_encode(s: str) -> str: ...
def json_str(s: str) -> str: ...
def csv_field(s: str) -> str: ...
def md_inline(s: str) -> str: ...
def md_render(s: str) -> str: ...
def html_page(title: str, body: str) -> str: ...
def dash_if_empty(s: str) -> str: ...
def labels_display(names: list[str]) -> str: ...
def filter_query(f: Filter, page: int) -> str: ...
def render_home(projects: list[Project]) -> str: ...
def render_users(users: list[User]) -> str: ...
def render_issue_row(i: Issue) -> str: ...
def render_pager(p: Project, f: Filter, pg: IssuePage) -> str: ...
def render_project(p: Project, labels: list[Label], f: Filter, pg: IssuePage) -> str: ...
def render_issue(i: Issue, comments: list[Comment], activity: list[Activity]) -> str: ...
def issue_line(i: Issue) -> str: ...
def issues_text(xs: list[Issue]) -> str: ...
def users_text(xs: list[User]) -> str: ...
def projects_text(xs: list[Project]) -> str: ...
def labels_text(xs: list[Label]) -> str: ...
def comments_text(xs: list[Comment]) -> str: ...
def activity_line(a: Activity) -> str: ...
def activity_text(xs: list[Activity]) -> str: ...
def issue_json(i: Issue, comments: list[Comment], activity: list[Activity]) -> str: ...
def issues_csv(xs: list[Issue]) -> str: ...
```

### 3.4 `04_routes` (Overlang `src/04_routes.ov`, Python `main.py`)

**Shims.** Of `runtime/ext/ov_http.h`: `ov_http_listen`, `ov_http_accept`,
`ov_http_method`, `ov_http_path`, `ov_http_query`, `ov_http_body`,
`ov_http_respond`, `ov_http_form_value` (`ov_http_html_escape` is bound in
`03_render`). No new HTTP shim is needed. Python uses `http.server` as the
todo port does (`HTTPServer`, a `BaseHTTPRequestHandler` that builds a
`Request` from `self.command`, `self.path` split at the first `?`, and the
`Content-Length` body decoded as UTF-8 with `errors="replace"`, then calls
`route` and writes the `Response` with Content-Length and `Connection: close`;
`log_message` silenced), and `form_value` via `urllib.parse.parse_qsl(body, keep_blank_values=True)`, first match.

```overlang
type Request = { method: Str, path: Str, query: Str, body: Str }
type Response = { status: Int, content_type: Str, body: Str }

foreign c "ov_http.h" fn http_listen(port: Int) -> Int !net = "ov_http_listen"
foreign c "ov_http.h" fn http_accept(server: Int) -> Int !net = "ov_http_accept"
foreign c "ov_http.h" fn http_method(req: Int) -> Str !net = "ov_http_method"
foreign c "ov_http.h" fn http_path(req: Int) -> Str !net = "ov_http_path"
foreign c "ov_http.h" fn http_query(req: Int) -> Str !net = "ov_http_query"
foreign c "ov_http.h" fn http_body(req: Int) -> Str !net = "ov_http_body"
foreign c "ov_http.h" fn http_respond(req: Int, status: Int, content_type: Str, body: Str) -> Int !net = "ov_http_respond"
foreign c "ov_http.h" fn form_value(body: Str, key: Str) -> Str = "ov_http_form_value"

// Section 2.1 constructors: 200 with the content type of the kind.
declare fn resp_html(body: Str) -> Response
declare fn resp_text(body: Str) -> Response
declare fn resp_json(body: Str) -> Response
declare fn resp_csv(body: Str) -> Response
// status, "text/plain; charset=utf-8", msg ++ "\n".
declare fn resp_error(status: Int, msg: Str) -> Response
// "\r\n" and "\r" -> "\n"; "\u{1e}" and "\u{1f}" -> " "; then trim.
declare fn clean_text(s: Str) -> Str
// "\r", "\n", "\u{1e}", "\u{1f}" -> " "; then trim.
declare fn clean_line(s: Str) -> Str
// clean_line(form_value(body, key)).
declare fn form_line(body: Str, key: Str) -> Str
// clean_text(form_value(body, key)).
declare fn form_text(body: Str, key: Str) -> Str
// 1 to 9 ASCII digits: their value; anything else (empty, sign, space, letters): -1.
declare fn parse_nat(s: Str) -> Int
  ensures result >= -1
// If seg ends with suffix and the rest parses with parse_nat to n >= 1: n; else 0. ("12.json", ".json") is 12.
declare fn issue_ref(seg: Str, suffix: Str) -> Int
  ensures result >= 0
// split(path, "/") without empty segments: "/p/WEB/3/" is ["p", "WEB", "3"]; "/" is [].
declare fn path_segments(path: Str) -> List[Str]
// Section 2.2: Ok(filter) or Err("bad status" | "bad priority" | "bad page").
declare fn parse_filter(query: Str) -> Result[Filter, Str]
// split(s, ","), each trimmed, empty pieces dropped, order kept.
declare fn parse_label_names(s: Str) -> List[Str]
// find_project; None gives 404 "no such project: " ++ key; else f(p).
declare fn with_project(db: Int, key: Str, f: fn(Project) -> Response !fs) -> Response !fs
// with_project, then find_issue; None gives 404 "no such issue: " ++ key ++ "-" ++ show(n); else f(p, i).
declare fn with_issue(db: Int, key: Str, n: Int, f: fn(Project, Issue) -> Response !fs) -> Response !fs
// Re-reads the issue and answers its HTML page (404 as with_issue if it is gone).
declare fn respond_issue(db: Int, p: Project, number: Int) -> Response !fs
// HTML project page for f.
declare fn respond_project(db: Int, p: Project, f: Filter) -> Response !fs
// One handler per route of section 2.2 (number in brackets); n is the issue number from issue_ref.
declare fn h_home(db: Int) -> Response !fs                                                  // [1]
declare fn h_health() -> Response                                                           // [2]
declare fn h_users_page(db: Int) -> Response !fs                                            // [3]
declare fn h_users_text(db: Int) -> Response !fs                                            // [4]
declare fn h_create_user(db: Int, body: Str) -> Response !fs                                // [5]
declare fn h_projects_text(db: Int) -> Response !fs                                         // [6]
declare fn h_create_project(db: Int, body: Str) -> Response !fs                             // [7]
declare fn h_project_page(db: Int, key: Str, query: Str) -> Response !fs                    // [8]
declare fn h_issues_text(db: Int, key: Str, query: Str) -> Response !fs                     // [9]
declare fn h_issues_csv(db: Int, key: Str) -> Response !fs                                  // [10]
declare fn h_create_issue(db: Int, key: Str, body: Str) -> Response !fs                     // [11]
declare fn h_labels_text(db: Int, key: Str) -> Response !fs                                 // [12]
declare fn h_create_label(db: Int, key: Str, body: Str) -> Response !fs                     // [13]
declare fn h_issue_page(db: Int, key: Str, n: Int) -> Response !fs                          // [14]
declare fn h_issue_json(db: Int, key: Str, n: Int) -> Response !fs                          // [15]
declare fn h_issue_text(db: Int, key: Str, n: Int) -> Response !fs                          // [16]
declare fn h_comments_text(db: Int, key: Str, n: Int) -> Response !fs                       // [17]
declare fn h_activity_text(db: Int, key: Str, n: Int) -> Response !fs                       // [18]
declare fn h_edit_issue(db: Int, key: Str, n: Int, body: Str) -> Response !fs               // [19]
declare fn h_set_status(db: Int, key: Str, n: Int, body: Str) -> Response !fs               // [20]
declare fn h_assign(db: Int, key: Str, n: Int, body: Str) -> Response !fs                   // [21]
declare fn h_add_comment(db: Int, key: Str, n: Int, body: Str) -> Response !fs              // [22]
declare fn h_add_label(db: Int, key: Str, n: Int, body: Str) -> Response !fs                // [23]
declare fn h_remove_label(db: Int, key: Str, n: Int, body: Str) -> Response !fs             // [24]
// match (req.method, path_segments(req.path)) to a handler; anything else 404 "not found" [25].
declare fn route(db: Int, req: Request) -> Response !fs
// argv without the program name: [db_path, port]; defaults "tracker.db" and 8080 (also for a bad port).
declare fn parse_args(argv: List[Str]) -> (Str, Int)
// Forever: accept (skip negative handles), build the Request, route, respond.
declare fn serve(db: Int, server: Int) -> Unit !net !fs
// parse_args(args()), db_open, http_listen, print section 2's messages (flush() after "listening"), serve.
declare fn main() -> Unit !io !net !fs !exit
```

Routing sketch (Overlang; the Python `match` is the same shape). Segment
patterns that end in a fixed name come before the `["p", key, n]` catch-all:

```
match (req.method, path_segments(req.path)) {
  ("GET", []) => h_home(db),
  ("GET", ["p", key, "issues.txt"]) => h_issues_text(db, key, req.query),
  ("GET", ["p", key, n]) if issue_ref(n, ".json") > 0 => h_issue_json(db, key, issue_ref(n, ".json")),
  ...
  _ => resp_error(404, "not found"),
}
```

```python
@dataclass(frozen=True)
class Request: method: str; path: str; query: str; body: str
class Response: status: int; content_type: str; body: str
def form_value(body: str, key: str) -> str: ...
def resp_html(body: str) -> Response: ...
def resp_text(body: str) -> Response: ...
def resp_json(body: str) -> Response: ...
def resp_csv(body: str) -> Response: ...
def resp_error(status: int, msg: str) -> Response: ...
def clean_text(s: str) -> str: ...
def clean_line(s: str) -> str: ...
def form_line(body: str, key: str) -> str: ...
def form_text(body: str, key: str) -> str: ...
def parse_nat(s: str) -> int: ...
def issue_ref(seg: str, suffix: str) -> int: ...
def path_segments(path: str) -> list[str]: ...
def parse_filter(query: str) -> tuple[Optional[Filter], str]: ...
def parse_label_names(s: str) -> list[str]: ...
def with_project(db: Db, key: str, f: Callable[[Project], Response]) -> Response: ...
def with_issue(db: Db, key: str, n: int, f: Callable[[Project, Issue], Response]) -> Response: ...
def respond_issue(db: Db, p: Project, number: int) -> Response: ...
def respond_project(db: Db, p: Project, f: Filter) -> Response: ...
def h_home(db: Db) -> Response: ...
def h_health() -> Response: ...
def h_users_page(db: Db) -> Response: ...
def h_users_text(db: Db) -> Response: ...
def h_create_user(db: Db, body: str) -> Response: ...
def h_projects_text(db: Db) -> Response: ...
def h_create_project(db: Db, body: str) -> Response: ...
def h_project_page(db: Db, key: str, query: str) -> Response: ...
def h_issues_text(db: Db, key: str, query: str) -> Response: ...
def h_issues_csv(db: Db, key: str) -> Response: ...
def h_create_issue(db: Db, key: str, body: str) -> Response: ...
def h_labels_text(db: Db, key: str) -> Response: ...
def h_create_label(db: Db, key: str, body: str) -> Response: ...
def h_issue_page(db: Db, key: str, n: int) -> Response: ...
def h_issue_json(db: Db, key: str, n: int) -> Response: ...
def h_issue_text(db: Db, key: str, n: int) -> Response: ...
def h_comments_text(db: Db, key: str, n: int) -> Response: ...
def h_activity_text(db: Db, key: str, n: int) -> Response: ...
def h_edit_issue(db: Db, key: str, n: int, body: str) -> Response: ...
def h_set_status(db: Db, key: str, n: int, body: str) -> Response: ...
def h_assign(db: Db, key: str, n: int, body: str) -> Response: ...
def h_add_comment(db: Db, key: str, n: int, body: str) -> Response: ...
def h_add_label(db: Db, key: str, n: int, body: str) -> Response: ...
def h_remove_label(db: Db, key: str, n: int, body: str) -> Response: ...
def route(db: Db, req: Request) -> Response: ...
def parse_args(argv: list[str]) -> tuple[str, int]: ...
def serve(db: Db, server: HTTPServer) -> None: ...
def main() -> None: ...                       # sys.exit(1) on the two startup failures
```

## 4. Files and assembly

| Builder | Overlang | Python |
|---|---|---|
| db | `examples/tracker/src/01_db.ov`, `runtime/ext/ov_sqlite.h`, `runtime/ext/ov_sqlite.c`, `runtime/ext/tests/test_sqlite.c` | `examples/tracker_py/db.py` |
| models | `examples/tracker/src/02_models.ov` | `examples/tracker_py/models.py` |
| render | `examples/tracker/src/03_render.ov` | `examples/tracker_py/render.py` |
| routes | `examples/tracker/src/04_routes.ov`, `examples/tracker/smoke.sh` | `examples/tracker_py/main.py`, `examples/tracker_py/test_tracker.py` |

All paths are under `/home/ec2-user/claude/overlang`. Each `.ov` file holds
its module's definitions and its `test` blocks; nothing else. Assembly, from
`examples/tracker` (scratch databases and build output stay under the
example directory, never `/tmp`):

```sh
ov=../../target/release/ov
$ov check src/01_db.ov src/02_models.ov src/03_render.ov src/04_routes.ov
$ov test  src/01_db.ov src/02_models.ov src/03_render.ov src/04_routes.ov
$ov build src/01_db.ov src/02_models.ov src/03_render.ov src/04_routes.ov -o tracker
# store path: rm -rf .ovstore; $ov add src/01_db.ov; $ov add src/02_models.ov; $ov add src/03_render.ov; $ov add src/04_routes.ov; $ov test
./smoke.sh ./tracker
cd ../tracker_py && python3 -m unittest -v test_tracker && ../tracker/smoke.sh python3 main.py
```

## 5. Tests

Overlang tests are `test` blocks in the module's own file; tests needing a
database open `db_open(":memory:")` (Python: `db_open(":memory:")`), so
nothing is written to disk. Python tests are in `test_tracker.py`, one
`unittest.TestCase` class per module (`TestDb`, `TestModels`, `TestRender`,
`TestRoutes`), with one method per Overlang test (`t_db_parse_table` becomes
`test_db_parse_table`), asserting the same values. String literals below are
Overlang; in Python write `\u{1f}` as `\x1f` and `\u{1e}` as `\x1e`.

### 5.1 `01_db` (4 tests)

- `t_db_parse_table`: `parse_table("a\u{1f}b\u{1e}1\u{1f}x\u{1e}2\u{1f}") == [["1", "x"], ["2", ""]]`;
  `parse_table("a\u{1f}b") == []`; `parse_table("") == []`;
  `cell_str(["a"], 3) == ""`; `cell_int(["7", "x"], 0) == 7`; `cell_int(["7", "x"], 1) == 0`.
- `t_db_params`: `sql_params([p_int(5), p_str("a b"), p_null(), p_int(-2)]) == "i5\u{1f}sa b\u{1f}n\u{1f}i-2"`; `sql_params([]) == ""`.
- `t_db_roundtrip`: `db_open(":memory:")` is >= 1 (Python: not None);
  `db_exec(db, "insert into users (name) values (?)", [p_str("ann")]) == 1`;
  `db_rows(db, "select id, name from users", []) == [["1", "ann"]]`;
  `db_int(db, "select count(*) from users where name = ?", [p_str("ann")]) == 1`;
  `db_rows(db, "select ? is null, ? + 1", [p_null(), p_int(41)]) == [["1", "42"]]`;
  `db_exec(db, "insert into nope values (1)", []) == -1`; `db_rows(db, "select * from nope", []) == []`;
  `db_exec(db, "select ?", ["x1"]) == -1` (bad tag).
- `t_db_schema`: after `db_open(":memory:")`, `db_rows(db, "select name from sqlite_master where type = 'table' and name != 'sqlite_sequence' order by name", [])`
  is `[["activity"], ["comments"], ["issue_labels"], ["issues"], ["labels"], ["projects"], ["users"]]`;
  a second `sqlite_exec(db, schema_sql())` returns 0 (the schema is idempotent);
  `sqlite_query_p(db, "select 1; select 2", "")` is `Some("1\u{1e}1")` (first statement only; Python `"1\x1e1"`).

The db builder also adds to `runtime/ext/tests/test_sqlite.c`: exec_p insert
with `"i1\x1fsx\x1fn"` changes 1 row; query_p returns the row with NULL as
empty; `"q1"`, `"i1x"`, `""` field (`"i1\x1f"`) and 33 fields each give -1 or
NULL with error `bad parameter`.

### 5.2 `02_models` (5 tests)

Fixture used by several tests (write it as a private helper, `m_fixture(db)`
/ `_fixture(db)`, or inline): users `alice` (id 1) and `bob` (id 2), project
`WEB` "Web site" (id 1), labels `bug` and `ui` in WEB.

- `t_m_validators`: `valid_project_key`: `"WEB"` and `"W3B"` true; `"W"`, `"web"`, `"3WB"`, `"ABCDEFGHIJK"` false.
  `valid_user_name`: `"alice"`, `"a.b-c_1"` true; `""`, `"a b"`, 33 `a`s false.
  `valid_label_name`: `"bug-1"` true; `"Bug"`, `""` false. `valid_status("open")` true, `valid_status("Open")` false.
- `t_m_pure_helpers`: `like_pattern("50%_a\\b") == "%50\\%\\_a\\\\b%"`; `page_count(0) == 1`;
  `page_count(20) == 1`; `page_count(21) == 2`; `page_offset(3) == 40`;
  `issue_where(no_filter()) == ("", [])`;
  `issue_where(Filter { ..no_filter(), status: "open", priority: 2 }) == (" and i.status = ? and i.priority = ?", ["sopen", "i2"])`.
- `t_m_issue_crud`: fixture; `create_issue(db, 1, "Fix login", "body", 2, 1, [bug_id, ui_id]) == 1`;
  `find_issue(db, 1, 1)` is `Some(Issue { id: 1, project_id: 1, project_key: "WEB", number: 1, title: "Fix login", body: "body", status: "open", priority: 2, assignee: "alice", labels: ["bug", "ui"] })`;
  a second issue gets number 2; an issue in a second project `API` gets number 1;
  `find_issue(db, 1, 99)` is None; `issue_key` of the first is `"WEB-1"`;
  `find_user(db, "nobody")` and `find_project(db, "NOPE")` are None.
- `t_m_search`: fixture; create 25 issues in WEB titled `"issue 1"` .. `"issue 25"`,
  priority `1 + k % 4` for issue k, issues 1 to 5 labelled `bug`, issue 7 with body `"50% done"`;
  close issues 1 to 3 with `set_status`. Then: `search_issues(no_filter())` has 20 issues, total 25,
  pages 2, first number 1; page 2 has 5 issues, first number 21;
  status `"closed"` gives total 3; label `"bug"` gives total 5; priority 1 gives total 6
  (k = 4, 8, ..., 24); q `"%"` gives exactly issue 7 (the `%` is literal); q `"ISSUE 2"`
  gives total 7 (2, 20..25; LIKE is ASCII case-insensitive); assignee `"alice"` gives total 0;
  `count_issues` agrees with `total`; `all_issues` has 25.
- `t_m_activity`: fixture; one issue; `set_status(db, i, "closed")` is true; re-read, `set_status(db, i2, "closed")` is false;
  `set_assignee(db, i2, find_user(db, "bob"))` true; `add_comment(db, i3, bob, "hi")` > 0; then
  `map(list_activity(db, i.id), |a| a.kind ++ ":" ++ a.detail) == ["created:", "status:closed", "assigned:bob", "commented:bob"]`;
  `list_comments` is `[Comment { id: 1, issue_id: 1, author: "bob", body: "hi" }]`;
  `set_assignee(db, i4, None)` true and logs `assigned:-`; `add_label` twice returns 1 then 0; `remove_label` returns 1.

### 5.3 `03_render` (5 tests, all pure)

Sample values: `iss = Issue { id: 1, project_id: 1, project_key: "WEB", number: 7, title: "Fix \"login\"", body: "line1\nline2", status: "open", priority: 2, assignee: "alice", labels: ["bug", "ui"] }`,
`web = Project { id: 1, key: "WEB", name: "Web <site>" }`.

- `t_r_markdown`: every row of the table in section 2.5.
- `t_r_text_json`: `issue_line(iss) == "WEB-7 open P2 alice [bug,ui] Fix \"login\"\n"`;
  `issue_line(Issue { ..iss, assignee: "", labels: [] })` is `"WEB-7 open P2 - [] Fix \"login\"\n"`;
  `activity_text([Activity { id: 1, issue_id: 1, kind: "created", detail: "" }, Activity { id: 2, issue_id: 1, kind: "status", detail: "closed" }]) == "created\nstatus closed\n"`;
  `comments_text([Comment { id: 1, issue_id: 1, author: "bob", body: "a\nb" }]) == "bob: a b\n"`;
  `json_str("a\"b\\c\nd\u{1}é") == "\"a\\\"b\\\\c\\nd\\u0001é\""`;
  `issue_json(Issue { ..iss, number: 1, title: "Say \"hi\"", assignee: "" }, [Comment { id: 1, issue_id: 1, author: "bob", body: "ok" }], [Activity { id: 1, issue_id: 1, kind: "created", detail: "" }])`
  equals the JSON example of section 2.3 followed by `\n`.
- `t_r_csv`: `csv_field("plain") == "plain"`; `csv_field("a,b") == "\"a,b\""`; `csv_field("say \"hi\"") == "\"say \"\"hi\"\"\""`;
  `issues_csv([Issue { ..iss, number: 1, title: "a,b", body: "x\ny", status: "open", priority: 3, assignee: "" }]) == "key,title,status,priority,assignee,labels,body\nWEB-1,\"a,b\",open,3,,bug;ui,\"x\ny\"\n"`;
  `issues_csv([])` is the header line alone.
- `t_r_query`: `url_encode("a b&c/é~") == "a%20b%26c%2F%C3%A9~"`;
  `filter_query(Filter { status: "open", assignee: "", label: "bug", priority: 2, q: "a b", page: 1 }, 3) == "status=open&label=bug&priority=2&q=a%20b&page=3"`;
  `filter_query(no_filter(), 2) == "page=2"`.
- `t_r_pages`: `render_issue(Issue { ..iss, title: "<x>" }, [], [])` contains `"<h1>WEB-7: &lt;x&gt;</h1>"` and not `"<x>"`,
  contains `"action=\"/p/WEB/7/status\""` and `"<button>close</button>"`;
  `render_project(web, [], no_filter(), IssuePage { issues: [iss], total: 41, page: 2, pages: 3 })` contains
  `"<h1>WEB: Web &lt;site&gt;</h1>"`, `"Page 2 of 3 (41 issues)"`, `"href=\"/p/WEB?page=1\">prev</a>"`,
  `"href=\"/p/WEB?page=3\">next</a>"`, `"<a href=\"/p/WEB/7\">WEB-7</a>"`, `"action=\"/p/WEB/issues\""`;
  `render_home([web])` contains `"<li><a href=\"/p/WEB\">WEB</a> Web &lt;site&gt;</li>"`;
  `html_page("a&b", "<i>x</i>")` contains `"<title>a&amp;b</title>"` and `"<i>x</i></body></html>"`.

### 5.4 `04_routes` (5 tests)

Helper (private): `rt_req(m, p, q, b)` builds a `Request` (Python `_req`).

- `t_rt_parsing`: `path_segments("/p/WEB/3/") == ["p", "WEB", "3"]`; `path_segments("/") == []`;
  `parse_nat("42") == 42`; `parse_nat("")`, `parse_nat("-1")`, `parse_nat("1x")`, `parse_nat("1234567890")` are -1;
  `issue_ref("12.json", ".json") == 12`; `issue_ref("12", "") == 12`; `issue_ref("0", "") == 0`;
  `issue_ref("12.json", "") == 0`; `clean_text("a\r\nb\u{1f}c ") == "a\nb c"`; `clean_line(" a\nb ") == "a b"`;
  `parse_label_names(" bug, ,ui ") == ["bug", "ui"]`.
- `t_rt_filter`: `parse_filter("status=open&priority=2&q=a+b&page=3")` is `Ok(Filter { status: "open", assignee: "", label: "", priority: 2, q: "a b", page: 3 })`;
  `parse_filter("")` is `Ok(no_filter())`; `parse_filter("status=done")` is `Err("bad status")`;
  `parse_filter("priority=5")` and `parse_filter("priority=x")` are `Err("bad priority")`;
  `parse_filter("page=0")` is `Err("bad page")`.
- `t_rt_unknown_404`: with db `0` (Python `None`): `route(0, rt_req("GET", "/nope", "", ""))` is
  `Response { status: 404, content_type: "text/plain; charset=utf-8", body: "not found\n" }`;
  `("DELETE", "/")` and `("POST", "/users.txt")` are 404 too; `("GET", "/health")` is 200 `"ok\n"`.
- `t_rt_flow`: `db = db_open(":memory:")`; POST `/users` `name=alice` is 200 and contains `<li>alice</li>`;
  again is 422 `"user exists\n"`; `name=` is 422 `"name required\n"`; `name=a+b` is 422 `"bad name\n"`;
  POST `/projects` `key=WEB&name=Web+site` is 200 containing `<h1>WEB: Web site</h1>`;
  POST `/p/WEB/labels` `name=bug` is 200; POST `/p/WEB/issues` `title=Fix+it&labels=bug&assignee=alice&priority=2` is 200
  containing `<h1>WEB-1: Fix it</h1>`; GET `/p/WEB/issues.txt` body is `"WEB-1 open P2 alice [bug] Fix it\n"`;
  POST `/p/WEB/1/status` `status=closed` then GET `/p/WEB/1/activity.txt` is `"created\nstatus closed\n"`;
  GET `/p/WEB/1.json` has content type `application/json` and starts with `{"key":"WEB-1",`;
  GET `/p/NOPE` is 404 `"no such project: NOPE\n"`; GET `/p/WEB/9` is 404 `"no such issue: WEB-9\n"`;
  GET `/p/WEB` with query `page=0` is 400 `"bad page\n"`; POST `/p/WEB/issues` `title=x&priority=9` is 422 `"bad priority\n"`,
  `title=x&priority=abc` is 400 `"bad priority\n"`, `title=x&labels=nope` is 422 `"no such label: nope\n"`.
- `t_rt_args`: `parse_args([]) == ("tracker.db", 8080)`; `parse_args(["x.db"]) == ("x.db", 8080)`;
  `parse_args(["x.db", "9000"]) == ("x.db", 9000)`; `parse_args(["x.db", "nine"]) == ("x.db", 8080)`.

### 5.5 Fuzzing

`ov test` fuzzes every pure function with a contract (`page_count`,
`page_offset`, `parse_nat`, `issue_ref`); keep those contracts exactly as
written so they hold for every input the fuzzer can generate. Effectful
functions with contracts are checked at run time only. Do not add `ensures`
clauses to other pure functions unless they hold for arbitrary strings.

### 5.6 Smoke script (`examples/tracker/smoke.sh`)

`smoke.sh <command...>` starts `<command...> <db> <port>` and checks it with
curl, in the style of `examples/todo/test.sh` (`set -euo pipefail`, a work
directory from `mktemp -d "$here/.tmp/run.XXXXXX"` removed on exit, a random
port in 20000..39999, poll `GET /health` up to 10 s, `fail` prints the server
log). It must pass unchanged for `./smoke.sh ./tracker` (from
`examples/tracker`) and `../tracker/smoke.sh python3 main.py` (from
`examples/tracker_py`). Helpers: `get PATH` prints the body, `code METHOD
PATH [DATA]` prints the status, `post PATH DATA` posts and discards. Steps
(each `expect` is an exact comparison or `grep -qF`):

1. Start on a fresh db. `get /health` is `ok`.
2. `post /users name=alice`, `post /users name=bob`; `get /users.txt` is `1 alice\n2 bob\n`.
   `code POST /users name=alice` is 422; `code POST /users name=` is 422.
3. `post /projects 'key=WEB&name=Web+site'`; `get /projects.txt` is `WEB Web site`;
   `code POST /projects 'key=web&name=x'` is 422; `get /` contains `<a href="/p/WEB">WEB</a>`.
4. `post /p/WEB/labels name=bug`, `post /p/WEB/labels name=ui`; `get /p/WEB/labels.txt` is `bug\nui`.
5. `post /p/WEB/issues 'title=Login+fails&body=Steps%3A+**click**+%60login%60&priority=1&assignee=alice&labels=bug,ui'`;
   `post /p/WEB/issues 'title=Typo+on+home&body=50%25+of+the+text'`;
   `get /p/WEB/issues.txt` is `WEB-1 open P1 alice [bug,ui] Login fails\nWEB-2 open P3 - [] Typo on home`.
6. `get /p/WEB/1` contains `<strong>click</strong>`, `<code>login</code>`, `<span class="status">open</span>`.
7. `post /p/WEB/1/status status=closed`; `post /p/WEB/2/assign assignee=bob`;
   `post /p/WEB/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E'`;
   `get /p/WEB/1/activity.txt` is `created\nstatus closed\ncommented bob`;
   `get /p/WEB/1/comments.txt` is `bob: Fixed in <b>v2</b>`; `get /p/WEB/1` contains `Fixed in &lt;b&gt;v2&lt;/b&gt;`.
8. Filters: `get '/p/WEB/issues.txt?status=open'` is `WEB-2 open P3 bob [] Typo on home`;
   `?label=bug` gives only `WEB-1`; `?assignee=bob` only `WEB-2`; `?priority=1` only `WEB-1`;
   `?q=50%25` only `WEB-2`; `?q=LOGIN` only `WEB-1`.
9. `get /p/WEB/1.json` is exactly
   `{"key":"WEB-1","project":"WEB","number":1,"title":"Login fails","body":"Steps: **click** `login`","status":"closed","priority":1,"assignee":"alice","labels":["bug","ui"],"comments":[{"author":"bob","body":"Fixed in <b>v2</b>"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"}]}`.
10. `get /p/WEB/issues.csv` first line is `key,title,status,priority,assignee,labels,body`, and it contains
    `WEB-2,Typo on home,open,3,bob,,50% of the text`.
11. Pagination: post 19 more issues (`title=n+$i`); `get /p/WEB/issues.txt | wc -l` is 20;
    `get '/p/WEB/issues.txt?page=2'` is `WEB-21 open P3 - [] n 19`; `get /p/WEB` contains `Page 1 of 2 (21 issues)`
    and `page=2">next</a>`.
12. Errors: `code GET /p/NOPE` 404 with body `no such project: NOPE`; `code GET /p/WEB/99` 404;
    `code GET '/p/WEB?page=abc'` 400; `code GET '/p/WEB?status=done'` 400;
    `code POST /p/WEB/issues title=` 422; `code POST /p/WEB/1/status status=done` 422;
    `code POST /p/WEB/1/labels label=nope` 422; `code GET /nope` 404; `code DELETE /` 404.
13. Restart on the same db; `get /p/WEB/issues.txt | head -1` is still `WEB-1 closed P1 alice [bug,ui] Login fails`.
    Print `PASS`.

(`get` output compared with `$(...)`, which drops the final newline, hence no
trailing `\n` in the expected values above.)
