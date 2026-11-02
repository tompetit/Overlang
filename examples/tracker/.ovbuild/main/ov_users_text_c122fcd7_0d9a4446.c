#include "ov_types.h"

ov_list* ov_map_9a617375_2918ff5c(ov_list*, ov_closure*);
ov_str* ov_users_text_c122fcd7_0d9a4446(ov_list* p0);

static ov_str* ovl_ov_users_text_c122fcd7_0d9a4446_4261(void* ovenv, ovt_User_9c127ab2* l4261_0) {
  int64_t t1 = l4261_0->f_id;
  ov_str* t2 = ov_show_int(t1);
  ov_str* t3 = ov_str_new(" ", 1);
  ov_str* t4 = ov_str_concat(t2, t3);
  ov_str* t5 = l4261_0->f_name;
  ov_str* t6 = ov_str_concat(t4, t5);
  ov_str* t7 = ov_str_new("\n", 1);
  ov_str* t8 = ov_str_concat(t6, t7);
  return t8;
}

ov_str* ov_users_text_c122fcd7_0d9a4446(ov_list* p0) {
  ov_closure* t9 = ov_closure_mk((void*)ovl_ov_users_text_c122fcd7_0d9a4446_4261, NULL);
  ov_list* t10 = ov_map_9a617375_2918ff5c(p0, t9);
  ov_str* t11 = ov_str_new("", 0);
  ov_str* t12 = ov_str_join(t10, t11);
  return t12;
}
