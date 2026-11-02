#include "ov_types.h"

ov_list* ov_list_activity_66c9a057_4d5ab5f1(int64_t, int64_t);
ov_list* ov_list_comments_625aa8cf_c111fe21(int64_t, int64_t);
ov_str* ov_issue_json_b69e0135_43ca48f1(ovt_Issue_b825796a*, ov_list*, ov_list*);
ovt_Response_dae56844* ov_resp_json_6a07290a_60bce079(ov_str*);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_issue_json_1ca34239_9a537815(int64_t p0, ov_str* p1, int64_t p2);

struct ovenv_5619 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_issue_json_1ca34239_9a537815_5619(void* ovenv, ovt_Project_73b03799* l5619_0, ovt_Issue_b825796a* l5619_1) {
  struct ovenv_5619* env = (struct ovenv_5619*)ovenv;
  int64_t p0 = env->p0;
  int64_t t1 = l5619_1->f_id;
  ov_list* t2 = ov_list_comments_625aa8cf_c111fe21(p0, t1);
  int64_t t3 = l5619_1->f_id;
  ov_list* t4 = ov_list_activity_66c9a057_4d5ab5f1(p0, t3);
  ov_str* t5 = ov_issue_json_b69e0135_43ca48f1(l5619_1, t2, t4);
  ovt_Response_dae56844* t6 = ov_resp_json_6a07290a_60bce079(t5);
  return t6;
}

ovt_Response_dae56844* ov_h_issue_json_1ca34239_9a537815(int64_t p0, ov_str* p1, int64_t p2) {
  struct ovenv_5619* t7 = GC_MALLOC(sizeof(struct ovenv_5619));
  t7->p0 = p0;
  ov_closure* t8 = ov_closure_mk((void*)ovl_ov_h_issue_json_1ca34239_9a537815_5619, t7);
  ovt_Response_dae56844* t9 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t8);
  return t9;
}
