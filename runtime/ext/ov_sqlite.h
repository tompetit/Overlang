/* SQLite shim for Overlang `foreign c`. Handles are small positive Ints; errors are negative Ints or NULL.
 * Strings returned stay valid forever (Boehm GC). Link with -lsqlite3 -lgc. */
#ifndef OV_SQLITE_H
#define OV_SQLITE_H

#include <stdint.h>

/* Open (creating if needed) the database file at path; returns a handle >= 1, or -1 on error (message: ov_sqlite_error(0)). */
int64_t ov_sqlite_open(const char* path);
/* Run one or more statements that return no rows; returns the number of rows changed, or -1 on error. */
int64_t ov_sqlite_exec(int64_t db, const char* sql);
/* Run a query and return all rows as text: first row is the column names, rows are separated by \x1e,
 * columns by \x1f, NULL is the empty string. Returns NULL on error. */
/* shortcut: values containing \x1e or \x1f are not escaped, add escaping if binary-ish text is stored */
const char* ov_sqlite_query(int64_t db, const char* sql);
/* Message of the last error on db (ov_sqlite_error(0) covers a failed open); "" if none. */
const char* ov_sqlite_error(int64_t db);
/* Close the database and free its handle slot; a bad handle is ignored. */
void ov_sqlite_close(int64_t db);

/* The bound variants run only the first statement of sql; ? parameters are numbered left to right. */

/* Like ov_sqlite_exec for a single statement with ? bound to the string p1. */
int64_t ov_sqlite_exec_1s(int64_t db, const char* sql, const char* p1);
/* Like ov_sqlite_exec for a single statement with ?1 and ?2 bound to the strings p1 and p2. */
int64_t ov_sqlite_exec_2s(int64_t db, const char* sql, const char* p1, const char* p2);
/* Like ov_sqlite_exec for a single statement with ? bound to the integer p1. */
int64_t ov_sqlite_exec_1i(int64_t db, const char* sql, int64_t p1);
/* Like ov_sqlite_query with ? bound to the string p1. */
const char* ov_sqlite_query_1s(int64_t db, const char* sql, const char* p1);
/* Like ov_sqlite_query with ? bound to the integer p1. */
const char* ov_sqlite_query_1i(int64_t db, const char* sql, int64_t p1);

#endif
