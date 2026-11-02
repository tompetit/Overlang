#include "ov_types.h"

int64_t ov_sqlite_exec_p_64951ad7_a19471ee(int64_t, ov_str*, ov_str*);
ov_str* ov_sql_params_f88159f9_4e330225(ov_list*);
int64_t ov_db_exec_3d590b5f_20515640(int64_t p0, ov_str* p1, ov_list* p2);

int64_t ov_db_exec_3d590b5f_20515640(int64_t p0, ov_str* p1, ov_list* p2) {
  ov_str* t1 = ov_sql_params_f88159f9_4e330225(p2);
  int64_t t2 = ov_sqlite_exec_p_64951ad7_a19471ee(p0, p1, t1);
  return t2;
}
