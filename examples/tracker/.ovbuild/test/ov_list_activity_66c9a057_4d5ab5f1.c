#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_86a0c9e5(ov_list*, ov_closure*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_Activity_dd4dbed0* ov_row_activity_752a75ce_287a9acd(ov_list*);
ov_list* ov_list_activity_66c9a057_4d5ab5f1(int64_t p0, int64_t p1);

static ovt_Activity_dd4dbed0* ovl_ov_list_activity_66c9a057_4d5ab5f1_2309(void* ovenv, ov_list* l2309_0) {
  ovt_Activity_dd4dbed0* t5 = ov_row_activity_752a75ce_287a9acd(l2309_0);
  return t5;
}

ov_list* ov_list_activity_66c9a057_4d5ab5f1(int64_t p0, int64_t p1) {
  ov_str* t1 = ov_str_new("select id, issue_id, kind, detail from activity where issue_id = \? order by id", 78);
  ov_str* t2 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t3 = ov_list_new(1);
  t3->items[0] = (ov_value){.p = t2};
  ov_list* t4 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t3);
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_list_activity_66c9a057_4d5ab5f1_2309, NULL);
  ov_list* t7 = ov_map_9a617375_86a0c9e5(t4, t6);
  return t7;
}
