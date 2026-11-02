#include "ov_types.h"

ov_list* ov_map_9a617375_bf19021d(ov_list*, ov_closure*);
ov_str* ov_labels_text_50fce26e_e183be1c(ov_list* p0);

static ov_str* ovl_ov_labels_text_50fce26e_e183be1c_4288(void* ovenv, ovt_Label_da07b2c4* l4288_0) {
  ov_str* t1 = l4288_0->f_name;
  ov_str* t2 = ov_str_new("\n", 1);
  ov_str* t3 = ov_str_concat(t1, t2);
  return t3;
}

ov_str* ov_labels_text_50fce26e_e183be1c(ov_list* p0) {
  ov_closure* t4 = ov_closure_mk((void*)ovl_ov_labels_text_50fce26e_e183be1c_4288, NULL);
  ov_list* t5 = ov_map_9a617375_bf19021d(p0, t4);
  ov_str* t6 = ov_str_new("", 0);
  ov_str* t7 = ov_str_join(t5, t6);
  return t7;
}
