#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
ovt_Option_7fe81862* ov_m_first_fb63178c_6c704d7e(ov_list*, ov_closure*);
ovt_User_9c127ab2* ov_row_user_e48587ac_cecb4b39(ov_list*);
ovt_Option_7fe81862* ov_find_user_4bac60c8_5f9c3bd0(int64_t p0, ov_str* p1);

static ovt_User_9c127ab2* ovl_ov_find_user_4bac60c8_5f9c3bd0_1800(void* ovenv, ov_list* l1800_0) {
  ovt_User_9c127ab2* t5 = ov_row_user_e48587ac_cecb4b39(l1800_0);
  return t5;
}

ovt_Option_7fe81862* ov_find_user_4bac60c8_5f9c3bd0(int64_t p0, ov_str* p1) {
  ov_str* t1 = ov_str_new("select id, name from users where name = \?", 41);
  ov_str* t2 = ov_p_str_73f85a82_17f92fe7(p1);
  ov_list* t3 = ov_list_new(1);
  t3->items[0] = (ov_value){.p = t2};
  ov_list* t4 = ov_db_rows_2c7fa5b5_89b40a99(p0, t1, t3);
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_find_user_4bac60c8_5f9c3bd0_1800, NULL);
  ovt_Option_7fe81862* t7 = ov_m_first_fb63178c_6c704d7e(t4, t6);
  return t7;
}
