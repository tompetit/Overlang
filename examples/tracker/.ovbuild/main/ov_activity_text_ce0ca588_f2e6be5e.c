#include "ov_types.h"

ov_list* ov_map_9a617375_7a67dbbf(ov_list*, ov_closure*);
ov_str* ov_activity_line_a767f335_9e5c839b(ovt_Activity_dd4dbed0*);
ov_str* ov_activity_text_ce0ca588_f2e6be5e(ov_list* p0);

static ov_str* ovl_ov_activity_text_ce0ca588_f2e6be5e_4335(void* ovenv, ovt_Activity_dd4dbed0* l4335_0) {
  ov_str* t1 = ov_activity_line_a767f335_9e5c839b(l4335_0);
  ov_str* t2 = ov_str_new("\n", 1);
  ov_str* t3 = ov_str_concat(t1, t2);
  return t3;
}

ov_str* ov_activity_text_ce0ca588_f2e6be5e(ov_list* p0) {
  ov_closure* t4 = ov_closure_mk((void*)ovl_ov_activity_text_ce0ca588_f2e6be5e_4335, NULL);
  ov_list* t5 = ov_map_9a617375_7a67dbbf(p0, t4);
  ov_str* t6 = ov_str_new("", 0);
  ov_str* t7 = ov_str_join(t5, t6);
  return t7;
}
