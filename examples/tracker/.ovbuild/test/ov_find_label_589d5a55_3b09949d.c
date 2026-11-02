#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
ovt_Label_da07b2c4* ov_row_label_0b6fd1dc_fa016a2c(ov_list*);
ovt_Option_c278faef* ov_m_first_fb63178c_4d2fb34d(ov_list*, ov_closure*);
ovt_Option_c278faef* ov_find_label_589d5a55_3b09949d(int64_t p0, int64_t p1, ov_str* p2);

static ovt_Label_da07b2c4* ovl_ov_find_label_589d5a55_3b09949d_1894(void* ovenv, ov_list* l1894_0) {
  ovt_Label_da07b2c4* t6 = ov_row_label_0b6fd1dc_fa016a2c(l1894_0);
  return t6;
}

ovt_Option_c278faef* ov_find_label_589d5a55_3b09949d(int64_t p0, int64_t p1, ov_str* p2) {
  ov_str* t1 = ov_str_new("select id, project_id, name from labels where project_id = \? and name = \?", 73);
  ov_str* t2 = ov_p_int_4640db5b_f21961d6(p1);
  ov_str* t3 = ov_p_str_73f85a82_17f92fe7(p2);
  ov_list* t4 = ov_list_new(2);
  t4->items[0] = (ov_value){.p = t2};
  t4->items[1] = (ov_value){.p = t3};
  ov_list* t5 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t4);
  ov_list* v1877 = t5;
  ov_closure* t7 = ov_closure_mk((void*)ovl_ov_find_label_589d5a55_3b09949d_1894, NULL);
  ovt_Option_c278faef* t8 = ov_m_first_fb63178c_4d2fb34d(v1877, t7);
  return t8;
}
