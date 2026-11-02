#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_2aeb273a(ov_list*, ov_closure*);
ov_str* ov_issue_select_sql_91164f3f_1b0791dc(void);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_Issue_b825796a* ov_row_issue_9a6bac26_e115135c(ov_list*);
ov_list* ov_all_issues_103a8b74_2317e431(int64_t p0, int64_t p1);

static ovt_Issue_b825796a* ovl_ov_all_issues_103a8b74_2317e431_2104(void* ovenv, ov_list* l2104_0) {
  ovt_Issue_b825796a* t7 = ov_row_issue_9a6bac26_e115135c(l2104_0);
  return t7;
}

ov_list* ov_all_issues_103a8b74_2317e431(int64_t p0, int64_t p1) {
  ov_str* t1 = ov_issue_select_sql_91164f3f_1b0791dc();
  ov_str* t2 = ov_str_new(" where i.project_id = \? order by i.number", 41);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t5 = ov_list_new(1);
  t5->items[0] = (ov_value){.p = t4};
  ov_list* t6 = ov_db_rows_2c7fa5b5_89b40a99(p0, t3, t5);
  ov_closure* t8 = ov_closure_mk((void*)ovl_ov_all_issues_103a8b74_2317e431_2104, NULL);
  ov_list* t9 = ov_map_9a617375_2aeb273a(t6, t8);
  return t9;
}
