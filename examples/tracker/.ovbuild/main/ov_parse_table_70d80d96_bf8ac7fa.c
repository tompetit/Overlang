#include "ov_types.h"

ov_list* ov_map_9a617375_f37f0510(ov_list*, ov_closure*);
ov_list* ov_parse_table_70d80d96_bf8ac7fa(ov_str* p0);

static ov_list* ovl_ov_parse_table_70d80d96_bf8ac7fa_1140(void* ovenv, ov_str* l1140_0) {
  ov_str* t4 = ov_str_new("\037", 1);
  ov_list* t5 = ov_str_split(l1140_0, t4);
  return t5;
}

ov_list* ov_parse_table_70d80d96_bf8ac7fa(ov_str* p0) {
  ov_str* t1 = ov_str_new("\036", 1);
  ov_list* t2 = ov_str_split(p0, t1);
  ov_list* t3 = ov_list_drop(t2, INT64_C(1));
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_parse_table_70d80d96_bf8ac7fa_1140, NULL);
  ov_list* t7 = ov_map_9a617375_f37f0510(t3, t6);
  return t7;
}
