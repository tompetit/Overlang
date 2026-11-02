/* SQLite shim: see ov_sqlite.h. Plain C, not dependent on rt.h. */
#include "ov_sqlite.h"

#include <gc.h>
#include <sqlite3.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>

/* shortcut: fixed table of 64 open databases, raise MAX_DB if an app needs more */
#define MAX_DB 64

static sqlite3* dbs[MAX_DB];
static char errs[MAX_DB + 1][256]; /* errs[0] is for failed opens, errs[h] for handle h */

/* A bound parameter: NULL when null is set, else a string, or an integer when s is NULL. */
typedef struct { const char* s; int64_t i; int null; } param;

static sqlite3* get(int64_t h) { return h >= 1 && h <= MAX_DB ? dbs[h - 1] : NULL; }

static void seterr(int64_t h, const char* msg) {
    strncpy(errs[h], msg, sizeof errs[h] - 1);
    errs[h][sizeof errs[h] - 1] = 0;
}

const char* ov_sqlite_error(int64_t db) {
    if (db < 0 || db > MAX_DB) return "bad handle";
    return errs[db];
}

int64_t ov_sqlite_open(const char* path) {
    for (int h = 1; h <= MAX_DB; h++) {
        if (dbs[h - 1]) continue;
        sqlite3* d = NULL;
        if (sqlite3_open(path, &d) != SQLITE_OK) {
            seterr(0, d ? sqlite3_errmsg(d) : "out of memory");
            sqlite3_close(d);
            return -1;
        }
        dbs[h - 1] = d;
        errs[h][0] = 0;
        return h;
    }
    seterr(0, "too many open databases");
    return -1;
}

void ov_sqlite_close(int64_t db) {
    sqlite3* d = get(db);
    if (!d) return;
    sqlite3_close(d);
    dbs[db - 1] = NULL;
    errs[db][0] = 0;
}

/* Growable GC byte buffer. */
typedef struct { char* p; size_t len, cap; } buf;

static void put(buf* b, const char* s, size_t n) {
    if (b->len + n + 1 > b->cap) {
        size_t nc = b->cap ? b->cap : 256;
        while (nc < b->len + n + 1) nc *= 2;
        char* np = GC_MALLOC_ATOMIC(nc);
        if (b->len) memcpy(np, b->p, b->len);
        b->p = np;
        b->cap = nc;
    }
    memcpy(b->p + b->len, s, n);
    b->len += n;
    b->p[b->len] = 0;
}

/* Prepare sql, bind np params, step to the end; with first set, only the first statement runs, else the whole
 * script does. With out != NULL, collect rows as text. Returns 0 or -1. */
static int run(int64_t h, const char* sql, const param* ps, int np, int first, buf* out, int64_t* changed) {
    sqlite3* d = get(h);
    if (!d) { if (h >= 0 && h <= MAX_DB) seterr(h, "database is closed"); return -1; }
    errs[h][0] = 0;
    int64_t before = sqlite3_total_changes(d);
    const char* tail = sql;
    int ok = 1;
    while (ok && *tail) {
        sqlite3_stmt* st = NULL;
        int rc = sqlite3_prepare_v2(d, tail, -1, &st, &tail);
        if (rc != SQLITE_OK) { ok = 0; break; }
        if (!st) continue; /* whitespace or comment only */
        for (int k = 0; k < np && ok; k++)
            ok = (ps[k].null ? sqlite3_bind_null(st, k + 1)
                  : ps[k].s ? sqlite3_bind_text(st, k + 1, ps[k].s, -1, SQLITE_TRANSIENT)
                          : sqlite3_bind_int64(st, k + 1, ps[k].i)) == SQLITE_OK;
        int ncol = sqlite3_column_count(st);
        if (ok && out && ncol > 0) {
            for (int c = 0; c < ncol; c++) {
                if (c) put(out, "\x1f", 1);
                const char* n = sqlite3_column_name(st, c);
                put(out, n ? n : "", n ? strlen(n) : 0);
            }
        }
        while (ok && (rc = sqlite3_step(st)) == SQLITE_ROW) {
            if (!out) continue;
            put(out, "\x1e", 1);
            for (int c = 0; c < ncol; c++) {
                if (c) put(out, "\x1f", 1);
                const char* t = (const char*)sqlite3_column_text(st, c); /* NULL -> empty */
                /* column_text is NUL-terminated; the result crosses as a C string, so a value is cut at its first NUL */
                put(out, t ? t : "", t ? strlen(t) : 0);
            }
        }
        if (ok && rc != SQLITE_DONE) ok = 0;
        sqlite3_finalize(st);
        if (first) break;
    }
    if (!ok) { seterr(h, sqlite3_errmsg(d)); return -1; }
    if (changed) *changed = sqlite3_total_changes(d) - before;
    return 0;
}

static int64_t do_exec(int64_t db, const char* sql, const param* ps, int np, int first) {
    int64_t n = 0;
    return run(db, sql, ps, np, first, NULL, &n) ? -1 : n;
}

static const char* do_query(int64_t db, const char* sql, const param* ps, int np, int first) {
    buf b = {0};
    if (run(db, sql, ps, np, first, &b, NULL)) return NULL;
    return b.p ? b.p : ""; /* a statement with no columns yields "" */
}

int64_t ov_sqlite_exec(int64_t db, const char* sql) { return do_exec(db, sql, NULL, 0, 0); }
int64_t ov_sqlite_exec_1s(int64_t db, const char* sql, const char* p1) {
    param ps[] = {{p1, 0, 0}};
    return do_exec(db, sql, ps, 1, 1);
}
int64_t ov_sqlite_exec_2s(int64_t db, const char* sql, const char* p1, const char* p2) {
    param ps[] = {{p1, 0, 0}, {p2, 0, 0}};
    return do_exec(db, sql, ps, 2, 1);
}
int64_t ov_sqlite_exec_1i(int64_t db, const char* sql, int64_t p1) {
    param ps[] = {{NULL, p1, 0}};
    return do_exec(db, sql, ps, 1, 1);
}
const char* ov_sqlite_query(int64_t db, const char* sql) { return do_query(db, sql, NULL, 0, 1); }
const char* ov_sqlite_query_1s(int64_t db, const char* sql, const char* p1) {
    param ps[] = {{p1, 0, 0}};
    return do_query(db, sql, ps, 1, 1);
}
const char* ov_sqlite_query_1i(int64_t db, const char* sql, int64_t p1) {
    param ps[] = {{NULL, p1, 0}};
    return do_query(db, sql, ps, 1, 1);
}

#define MAX_PARAMS 32

/* Decode params (see ov_sqlite.h) into ps, pointing into a GC copy; returns the count, or -1 (error set on h). */
static int decode(int64_t h, const char* params, param* ps) {
    int n = 0;
    if (*params) {
        size_t len = strlen(params);
        char* p = GC_MALLOC_ATOMIC(len + 1);
        memcpy(p, params, len + 1);
        for (char* f = p;; n++) {
            char* sep = strchr(f, '\x1f');
            if (sep) *sep = 0;
            if (n == MAX_PARAMS) goto bad;
            param* q = &ps[n];
            *q = (param){NULL, 0, 0};
            if (*f == 's') q->s = f + 1;
            else if (*f == 'n' && !f[1]) q->null = 1;
            else if (*f == 'i') {
                const char* d = f[1] == '-' ? f + 2 : f + 1;
                if (!*d || strspn(d, "0123456789") != strlen(d)) goto bad;
                char* end;
                errno = 0;
                q->i = strtoll(f + 1, &end, 10);
                if (errno || *end) goto bad;
            } else goto bad;
            if (!sep) { n++; break; }
            f = sep + 1;
        }
    }
    return n;
bad:
    if (h >= 0 && h <= MAX_DB) seterr(h, "bad parameter");
    return -1;
}

int64_t ov_sqlite_exec_p(int64_t db, const char* sql, const char* params) {
    param ps[MAX_PARAMS];
    int n = decode(db, params, ps);
    return n < 0 ? -1 : do_exec(db, sql, ps, n, 1);
}
const char* ov_sqlite_query_p(int64_t db, const char* sql, const char* params) {
    param ps[MAX_PARAMS];
    int n = decode(db, params, ps);
    return n < 0 ? NULL : do_query(db, sql, ps, n, 1);
}
