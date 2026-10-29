/* Tests for ov_sqlite: run from runtime/ext (the Makefile does). */
#include "ov_sqlite.h"

#include <gc.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

static int checks, fails;
#define CHECK(c) do { checks++; if (!(c)) { fails++; printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); } } while (0)
#define CHECK_STR(a, b) do { const char* a_ = (a); checks++; \
    if (!a_ || strcmp(a_, (b))) { fails++; printf("FAIL %s:%d: got \"%s\" want \"%s\"\n", __FILE__, __LINE__, a_ ? a_ : "(null)", (b)); } } while (0)

int main(void) {
    GC_INIT();
    const char* path = "tests/build/test.db";
    unlink(path);
    int64_t db = ov_sqlite_open(path);
    CHECK(db >= 1);
    CHECK(ov_sqlite_open("/nonexistent-dir/x.db") < 0);
    CHECK(strlen(ov_sqlite_error(0)) > 0);

    CHECK(ov_sqlite_exec(db, "CREATE TABLE todo(id INTEGER PRIMARY KEY, title TEXT, note TEXT, done INTEGER)") == 0);
    CHECK(ov_sqlite_exec_2s(db, "INSERT INTO todo(title, note, done) VALUES(?, ?, 0)", "it's a 'test'", "x\"y; DROP TABLE todo;--") == 1);
    CHECK(ov_sqlite_exec_1s(db, "INSERT INTO todo(title) VALUES(?)", "second") == 1);
    CHECK(ov_sqlite_exec(db, "INSERT INTO todo(title) VALUES('a'); INSERT INTO todo(title) VALUES('b')") == 2);

    CHECK_STR(ov_sqlite_query(db, "SELECT id, title, note FROM todo WHERE id <= 2 ORDER BY id"),
              "id\x1f" "title\x1f" "note\x1e" "1\x1f" "it's a 'test'\x1f" "x\"y; DROP TABLE todo;--\x1e" "2\x1f" "second\x1f");
    CHECK_STR(ov_sqlite_query_1i(db, "SELECT title FROM todo WHERE id = ?", 2), "title\x1e" "second");
    CHECK_STR(ov_sqlite_query_1s(db, "SELECT id FROM todo WHERE title = ?", "it's a 'test'"), "id\x1e" "1");
    CHECK_STR(ov_sqlite_query_1s(db, "SELECT id FROM todo WHERE title = ?", "nothing"), "id");
    CHECK_STR(ov_sqlite_query(db, "SELECT 1 WHERE 0"), "1");

    CHECK(ov_sqlite_exec_1i(db, "UPDATE todo SET done = 1 WHERE id <= ?", 2) == 2);
    CHECK(ov_sqlite_exec_1i(db, "DELETE FROM todo WHERE id = ?", 99) == 0);
    CHECK_STR(ov_sqlite_query(db, "SELECT count(*), sum(done) FROM todo"), "count(*)\x1f" "sum(done)\x1e" "4\x1f" "2");
    CHECK_STR(ov_sqlite_query(db, "SELECT 1 AS a, 2.5 AS b, 'é' AS c"), "a\x1f" "b\x1f" "c\x1e" "1\x1f" "2.5\x1f" "é");

    /* error paths */
    CHECK(ov_sqlite_query(db, "SELEC nope") == NULL);
    CHECK(strstr(ov_sqlite_error(db), "syntax error") != NULL);
    CHECK(ov_sqlite_exec(db, "INSERT INTO missing VALUES(1)") < 0);
    CHECK(strstr(ov_sqlite_error(db), "no such table") != NULL);
    CHECK(ov_sqlite_exec_1i(db, "INSERT INTO todo(id) VALUES(?)", 1) < 0); /* primary key clash */
    CHECK(strstr(ov_sqlite_error(db), "UNIQUE") != NULL);
    CHECK(ov_sqlite_query(db, "SELECT 1") != NULL);
    CHECK_STR(ov_sqlite_error(db), "");
    CHECK(ov_sqlite_exec(999, "SELECT 1") < 0);
    CHECK(ov_sqlite_query(0, "SELECT 1") == NULL);

    /* data survives close and reopen */
    ov_sqlite_close(db);
    ov_sqlite_close(db); /* double close is ignored */
    CHECK(ov_sqlite_exec(db, "SELECT 1") < 0);
    int64_t db2 = ov_sqlite_open(path);
    CHECK(db2 >= 1);
    CHECK_STR(ov_sqlite_query(db2, "SELECT count(*) FROM todo"), "count(*)\x1e" "4");
    ov_sqlite_close(db2);

    printf("test_sqlite: %d checks, %d failed\n", checks, fails);
    return fails ? 1 : 0;
}
