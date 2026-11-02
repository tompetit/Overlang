#include "ov_types.h"
#include "ov_sqlite.h"

int64_t ov_sqlite_exec_p_64951ad7_a19471ee(int64_t p0, ov_str* p1, ov_str* p2);

int64_t ov_sqlite_exec_p_64951ad7_a19471ee(int64_t p0, ov_str* p1, ov_str* p2) {
  return ov_sqlite_exec_p(p0, p1->data, p2->data);
}
