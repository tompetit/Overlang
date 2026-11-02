#include "ov_types.h"

ov_list* ov_filter_021377ec_06b1c4d5(ov_list*, ov_closure*);
ov_list* ov_path_segments_dbe7e6ce_5890a3e7(ov_str* p0);

static bool ovl_ov_path_segments_dbe7e6ce_5890a3e7_4958(void* ovenv, ov_str* l4958_0) {
  ov_str* t3 = ov_str_new("", 0);
  bool t4 = (!ov_str_eq(l4958_0, t3));
  return t4;
}

ov_list* ov_path_segments_dbe7e6ce_5890a3e7(ov_str* p0) {
  ov_str* t1 = ov_str_new("/", 1);
  ov_list* t2 = ov_str_split(p0, t1);
  ov_closure* t5 = ov_closure_mk((void*)ovl_ov_path_segments_dbe7e6ce_5890a3e7_4958, NULL);
  ov_list* t6 = ov_filter_021377ec_06b1c4d5(t2, t5);
  return t6;
}
