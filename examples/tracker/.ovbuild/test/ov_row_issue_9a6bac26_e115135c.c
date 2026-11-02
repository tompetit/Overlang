#include "ov_types.h"

int64_t ov_cell_int_241695c3_c8b376c8(ov_list*, int64_t);
ov_list* ov_filter_021377ec_06b1c4d5(ov_list*, ov_closure*);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list*, int64_t);
ovt_Issue_b825796a* ov_row_issue_9a6bac26_e115135c(ov_list* p0);

static bool ovl_ov_row_issue_9a6bac26_e115135c_1619(void* ovenv, ov_str* l1619_0) {
  ov_str* t13 = ov_str_new("", 0);
  bool t14 = (!ov_str_eq(l1619_0, t13));
  return t14;
}

ovt_Issue_b825796a* ov_row_issue_9a6bac26_e115135c(ov_list* p0) {
  int64_t t1 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(0));
  int64_t t2 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(1));
  ov_str* t3 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(2));
  int64_t t4 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(3));
  ov_str* t5 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(4));
  ov_str* t6 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(5));
  ov_str* t7 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(6));
  int64_t t8 = ov_cell_int_241695c3_c8b376c8(p0, INT64_C(7));
  ov_str* t9 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(8));
  ov_str* t10 = ov_cell_str_c5e3a532_bb12c045(p0, INT64_C(9));
  ov_str* t11 = ov_str_new(",", 1);
  ov_list* t12 = ov_str_split(t10, t11);
  ov_closure* t15 = ov_closure_mk((void*)ovl_ov_row_issue_9a6bac26_e115135c_1619, NULL);
  ov_list* t16 = ov_filter_021377ec_06b1c4d5(t12, t15);
  ov_list* t17 = ov_list_sort(t16, ov_cmp_str);
  ovt_Issue_b825796a* t18 = ovt_Issue_b825796a_mk(t1, t2, t3, t4, t5, t6, t7, t8, t9, t17);
  return t18;
}
