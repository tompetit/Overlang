#include "ov_types.h"

ov_list* ov_filter_021377ec_06b1c4d5(ov_list*, ov_closure*);
ov_list* ov_map_9a617375_6da7d9d8(ov_list*, ov_closure*);
ov_list* ov_parse_label_names_38f63ee7_6e2c6f9a(ov_str* p0);

static ov_str* ovl_ov_parse_label_names_38f63ee7_6e2c6f9a_5056(void* ovenv, ov_str* l5056_0) {
  ov_str* t3 = ov_str_trim(l5056_0);
  return t3;
}

static bool ovl_ov_parse_label_names_38f63ee7_6e2c6f9a_5061(void* ovenv, ov_str* l5061_0) {
  ov_str* t6 = ov_str_new("", 0);
  bool t7 = (!ov_str_eq(l5061_0, t6));
  return t7;
}

ov_list* ov_parse_label_names_38f63ee7_6e2c6f9a(ov_str* p0) {
  ov_str* t1 = ov_str_new(",", 1);
  ov_list* t2 = ov_str_split(p0, t1);
  ov_closure* t4 = ov_closure_mk((void*)ovl_ov_parse_label_names_38f63ee7_6e2c6f9a_5056, NULL);
  ov_list* t5 = ov_map_9a617375_6da7d9d8(t2, t4);
  ov_closure* t8 = ov_closure_mk((void*)ovl_ov_parse_label_names_38f63ee7_6e2c6f9a_5061, NULL);
  ov_list* t9 = ov_filter_021377ec_06b1c4d5(t5, t8);
  return t9;
}
