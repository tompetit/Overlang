#include "ov_types.h"

ov_str* ov_issue_line_3b908aae_6b62b185(ovt_Issue_b825796a*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_issue_text_d7b2d354_587a2631(int64_t p0, ov_str* p1, int64_t p2);

static ovt_Response_dae56844* ovl_ov_h_issue_text_d7b2d354_587a2631_5630(void* ovenv, ovt_Project_73b03799* l5630_0, ovt_Issue_b825796a* l5630_1) {
  ov_str* t1 = ov_issue_line_3b908aae_6b62b185(l5630_1);
  ovt_Response_dae56844* t2 = ov_resp_text_8a5de9a9_e3271fe2(t1);
  return t2;
}

ovt_Response_dae56844* ov_h_issue_text_d7b2d354_587a2631(int64_t p0, ov_str* p1, int64_t p2) {
  ov_closure* t3 = ov_closure_mk((void*)ovl_ov_h_issue_text_d7b2d354_587a2631_5630, NULL);
  ovt_Response_dae56844* t4 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t3);
  return t4;
}
