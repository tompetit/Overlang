#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
ovt_Option_da3cc0d5* ov_m_first_fb63178c_5c96a8df(ov_list*, ov_closure*);
ovt_Project_73b03799* ov_row_project_ca4a0485_7b4b99ad(ov_list*);
ovt_Option_da3cc0d5* ov_find_project_da7b7099_8f536708(int64_t p0, ov_str* p1);

static ovt_Project_73b03799* ovl_ov_find_project_da7b7099_8f536708_1847(void* ovenv, ov_list* l1847_0) {
  ovt_Project_73b03799* t5 = ov_row_project_ca4a0485_7b4b99ad(l1847_0);
  return t5;
}

ovt_Option_da3cc0d5* ov_find_project_da7b7099_8f536708(int64_t p0, ov_str* p1) {
  ov_str* t1 = ov_str_new("select id, key, name from projects where key = \?", 48);
  ov_str* t2 = ov_p_str_73f85a82_17f92fe7(p1);
  ov_list* t3 = ov_list_new(1);
  t3->items[0] = (ov_value){.p = t2};
  ov_list* t4 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t3);
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_find_project_da7b7099_8f536708_1847, NULL);
  ovt_Option_da3cc0d5* t7 = ov_m_first_fb63178c_5c96a8df(t4, t6);
  return t7;
}
