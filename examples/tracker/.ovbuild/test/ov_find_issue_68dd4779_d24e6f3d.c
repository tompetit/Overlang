#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_str* ov_issue_select_sql_91164f3f_1b0791dc(void);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_Issue_b825796a* ov_row_issue_9a6bac26_e115135c(ov_list*);
ovt_Option_c3749d6a* ov_m_first_fb63178c_576dd361(ov_list*, ov_closure*);
ovt_Option_c3749d6a* ov_find_issue_68dd4779_d24e6f3d(int64_t p0, int64_t p1, int64_t p2);

static ovt_Issue_b825796a* ovl_ov_find_issue_68dd4779_d24e6f3d_2001(void* ovenv, ov_list* l2001_0) {
  ovt_Issue_b825796a* t8 = ov_row_issue_9a6bac26_e115135c(l2001_0);
  return t8;
}

ovt_Option_c3749d6a* ov_find_issue_68dd4779_d24e6f3d(int64_t p0, int64_t p1, int64_t p2) {
  ov_str* t1 = ov_issue_select_sql_91164f3f_1b0791dc();
  ov_str* t2 = ov_str_new(" where i.project_id = \? and i.number = \?", 40);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = ov_p_int_4640db5b_f21961d6(p1);
  ov_str* t5 = ov_p_int_4640db5b_f21961d6(p2);
  ov_list* t6 = ov_list_new(2);
  t6->items[0] = (ov_value){.p = t4};
  t6->items[1] = (ov_value){.p = t5};
  ov_list* t7 = ov_db_rows_2c7fa5b5_89b40a99(p0, t3, t6);
  ov_list* v1981 = t7;
  ov_closure* t9 = ov_closure_mk((void*)ovl_ov_find_issue_68dd4779_d24e6f3d_2001, NULL);
  ovt_Option_c3749d6a* t10 = ov_m_first_fb63178c_576dd361(v1981, t9);
  return t10;
}
