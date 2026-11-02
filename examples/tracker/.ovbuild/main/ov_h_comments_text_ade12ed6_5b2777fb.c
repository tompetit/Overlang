#include "ov_types.h"

ov_list* ov_list_comments_625aa8cf_c111fe21(int64_t, int64_t);
ov_str* ov_comments_text_93076aed_8ec4b672(ov_list*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_comments_text_ade12ed6_5b2777fb(int64_t p0, ov_str* p1, int64_t p2);

struct ovenv_5645 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_comments_text_ade12ed6_5b2777fb_5645(void* ovenv, ovt_Project_73b03799* l5645_0, ovt_Issue_b825796a* l5645_1) {
  struct ovenv_5645* env = (struct ovenv_5645*)ovenv;
  int64_t p0 = env->p0;
  int64_t t1 = l5645_1->f_id;
  ov_list* t2 = ov_list_comments_625aa8cf_c111fe21(p0, t1);
  ov_str* t3 = ov_comments_text_93076aed_8ec4b672(t2);
  ovt_Response_dae56844* t4 = ov_resp_text_8a5de9a9_e3271fe2(t3);
  return t4;
}

ovt_Response_dae56844* ov_h_comments_text_ade12ed6_5b2777fb(int64_t p0, ov_str* p1, int64_t p2) {
  struct ovenv_5645* t5 = GC_MALLOC(sizeof(struct ovenv_5645));
  t5->p0 = p0;
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_h_comments_text_ade12ed6_5b2777fb_5645, t5);
  ovt_Response_dae56844* t7 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t6);
  return t7;
}
