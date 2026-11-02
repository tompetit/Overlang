#include "ov_types.h"

ov_list* ov_map_9a617375_6bc6a23d(ov_list*, ov_closure*);
ov_str* ov_projects_text_bab0fb0c_a03c894e(ov_list* p0);

static ov_str* ovl_ov_projects_text_bab0fb0c_a03c894e_4277(void* ovenv, ovt_Project_73b03799* l4277_0) {
  ov_str* t1 = l4277_0->f_key;
  ov_str* t2 = ov_str_new(" ", 1);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = l4277_0->f_name;
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_str* t6 = ov_str_new("\n", 1);
  ov_str* t7 = ov_str_concat(t5, t6);
  return t7;
}

ov_str* ov_projects_text_bab0fb0c_a03c894e(ov_list* p0) {
  ov_closure* t8 = ov_closure_mk((void*)ovl_ov_projects_text_bab0fb0c_a03c894e_4277, NULL);
  ov_list* t9 = ov_map_9a617375_6bc6a23d(p0, t8);
  ov_str* t10 = ov_str_new("", 0);
  ov_str* t11 = ov_str_join(t9, t10);
  return t11;
}
