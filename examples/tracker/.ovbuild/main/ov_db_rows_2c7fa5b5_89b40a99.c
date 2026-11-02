#include "ov_types.h"

ov_list* ov_parse_table_70d80d96_bf8ac7fa(ov_str*);
ov_str* ov_sql_params_f88159f9_4e330225(ov_list*);
ovt_Option_6133f15b* ov_sqlite_query_p_9c03480a_2da2338d(int64_t, ov_str*, ov_str*);
ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t p0, ov_str* p1, ov_list* p2);

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t p0, ov_str* p1, ov_list* p2) {
  ov_str* t1 = ov_sql_params_f88159f9_4e330225(p2);
  ovt_Option_6133f15b* t2 = ov_sqlite_query_p_9c03480a_2da2338d(p0, p1, t1);
  ov_list* t3 = 0; (void)t3;
  bool t4 = false; (void)t4;
  if (!t4) {
    if ((t2->tag == 0)) {
      ov_str* v1101 = t2->u.c0.f0;
      t4 = true;
      ov_list* t5 = ov_parse_table_70d80d96_bf8ac7fa(v1101);
      t3 = t5;
    }
  }
  if (!t4) {
    if ((t2->tag == 1)) {
      t4 = true;
      ov_list* t6 = ov_list_empty();
      t3 = t6;
    }
  }
  return t3;
}
