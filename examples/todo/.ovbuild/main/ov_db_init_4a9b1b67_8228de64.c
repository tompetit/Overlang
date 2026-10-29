#include "ov_types.h"

int64_t ov_sqlite_exec_ef173262_df21757d(int64_t, ov_str*);
int64_t ov_db_init_4a9b1b67_8228de64(int64_t p0);

int64_t ov_db_init_4a9b1b67_8228de64(int64_t p0) {
  ov_str* t1 = ov_str_new("create table if not exists todos (id integer primary key autoincrement, title text not null, done integer not null default 0)", 125);
  int64_t t2 = ov_sqlite_exec_ef173262_df21757d(p0, t1);
  return t2;
}
