#include "ov_types.h"
#include "ov_sqlite.h"

int64_t ov_sqlite_exec_ef173262_df21757d(int64_t p0, ov_str* p1);

int64_t ov_sqlite_exec_ef173262_df21757d(int64_t p0, ov_str* p1) {
  return ov_sqlite_exec(p0, p1->data);
}
