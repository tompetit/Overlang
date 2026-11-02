#include "ov_types.h"

ov_list* ov_list_activity_66c9a057_4d5ab5f1(int64_t, int64_t);
ov_list* ov_list_comments_625aa8cf_c111fe21(int64_t, int64_t);
ov_str* ov_render_issue_57e4bb34_d2db72ef(ovt_Issue_b825796a*, ov_list*, ov_list*);
ovt_Response_dae56844* ov_resp_html_e2b10d08_fbb70ecb(ov_str*);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_issue_page_d85f4494_98419fd6(int64_t p0, ov_str* p1, int64_t p2);

struct ovenv_5598 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_issue_page_d85f4494_98419fd6_5598(void* ovenv, ovt_Project_73b03799* l5598_0, ovt_Issue_b825796a* l5598_1) {
  struct ovenv_5598* env = (struct ovenv_5598*)ovenv;
  int64_t p0 = env->p0;
  int64_t t1 = l5598_1->f_id;
  ov_list* t2 = ov_list_comments_625aa8cf_c111fe21(p0, t1);
  int64_t t3 = l5598_1->f_id;
  ov_list* t4 = ov_list_activity_66c9a057_4d5ab5f1(p0, t3);
  ov_str* t5 = ov_render_issue_57e4bb34_d2db72ef(l5598_1, t2, t4);
  ovt_Response_dae56844* t6 = ov_resp_html_e2b10d08_fbb70ecb(t5);
  return t6;
}

ovt_Response_dae56844* ov_h_issue_page_d85f4494_98419fd6(int64_t p0, ov_str* p1, int64_t p2) {
  struct ovenv_5598* t7 = GC_MALLOC(sizeof(struct ovenv_5598));
  t7->p0 = p0;
  ov_closure* t8 = ov_closure_mk((void*)ovl_ov_h_issue_page_d85f4494_98419fd6_5598, t7);
  ovt_Response_dae56844* t9 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t8);
  return t9;
}
