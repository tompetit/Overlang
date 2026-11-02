#include "ov_types.h"

ov_list* ov_map_9a617375_4c51e675(ov_list*, ov_closure*);
ov_str* ov_issue_line_3b908aae_6b62b185(ovt_Issue_b825796a*);
ov_str* ov_issues_text_2e2d8929_b997bacc(ov_list* p0);

static ov_str* ovl_ov_issues_text_2e2d8929_b997bacc_4243(void* ovenv, ovt_Issue_b825796a* l4243_0) {
  ov_str* t1 = ov_issue_line_3b908aae_6b62b185(l4243_0);
  return t1;
}

ov_str* ov_issues_text_2e2d8929_b997bacc(ov_list* p0) {
  ov_closure* t2 = ov_closure_mk((void*)ovl_ov_issues_text_2e2d8929_b997bacc_4243, NULL);
  ov_list* t3 = ov_map_9a617375_4c51e675(p0, t2);
  ov_str* t4 = ov_str_new("", 0);
  ov_str* t5 = ov_str_join(t3, t4);
  return t5;
}
