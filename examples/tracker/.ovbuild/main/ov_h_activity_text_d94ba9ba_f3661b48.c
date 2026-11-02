#include "ov_types.h"

ov_list* ov_list_activity_66c9a057_4d5ab5f1(int64_t, int64_t);
ov_str* ov_activity_text_ce0ca588_f2e6be5e(ov_list*);
ovt_Response_dae56844* ov_resp_text_8a5de9a9_e3271fe2(ov_str*);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_activity_text_d94ba9ba_f3661b48(int64_t p0, ov_str* p1, int64_t p2);

struct ovenv_5660 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_activity_text_d94ba9ba_f3661b48_5660(void* ovenv, ovt_Project_73b03799* l5660_0, ovt_Issue_b825796a* l5660_1) {
  struct ovenv_5660* env = (struct ovenv_5660*)ovenv;
  int64_t p0 = env->p0;
  int64_t t1 = l5660_1->f_id;
  ov_list* t2 = ov_list_activity_66c9a057_4d5ab5f1(p0, t1);
  ov_str* t3 = ov_activity_text_ce0ca588_f2e6be5e(t2);
  ovt_Response_dae56844* t4 = ov_resp_text_8a5de9a9_e3271fe2(t3);
  return t4;
}

ovt_Response_dae56844* ov_h_activity_text_d94ba9ba_f3661b48(int64_t p0, ov_str* p1, int64_t p2) {
  struct ovenv_5660* t5 = GC_MALLOC(sizeof(struct ovenv_5660));
  t5->p0 = p0;
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_h_activity_text_d94ba9ba_f3661b48_5660, t5);
  ovt_Response_dae56844* t7 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t6);
  return t7;
}
