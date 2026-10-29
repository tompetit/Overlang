#include "ov_types.h"

ov_list* ov_parse_rows_e9206b62_50e2d628(ov_str*);
ovt_Option_6133f15b* ov_sqlite_query_0f96a2fa_d75bed94(int64_t, ov_str*);
ov_list* ov_list_todos_24d1401c_1ec84d65(int64_t p0);

ov_list* ov_list_todos_24d1401c_1ec84d65(int64_t p0) {
  ov_str* t1 = ov_str_new("select id, title, done from todos order by id", 45);
  ovt_Option_6133f15b* t2 = ov_sqlite_query_0f96a2fa_d75bed94(p0, t1);
  ov_list* t3 = 0; (void)t3;
  bool t4 = false; (void)t4;
  if (!t4) {
    if ((t2->tag == 0)) {
      ov_str* v409 = t2->u.c0.f0;
      t4 = true;
      ov_list* t5 = ov_parse_rows_e9206b62_50e2d628(v409);
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
