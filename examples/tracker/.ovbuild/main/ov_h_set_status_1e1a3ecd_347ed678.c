#include "ov_types.h"

bool ov_set_status_7fbb65d5_46ec26ca(int64_t, ovt_Issue_b825796a*, ov_str*);
bool ov_valid_status_8b9994c4_c3c13714(ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t, ovt_Project_73b03799*, int64_t);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_set_status_1e1a3ecd_347ed678(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3);

struct ovenv_5758 { ov_str* p3; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_set_status_1e1a3ecd_347ed678_5758(void* ovenv, ovt_Project_73b03799* l5758_0, ovt_Issue_b825796a* l5758_1) {
  struct ovenv_5758* env = (struct ovenv_5758*)ovenv;
  ov_str* p3 = env->p3;
  int64_t p0 = env->p0;
  ov_str* t1 = ov_str_new("status", 6);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p3, t1);
  ov_str* v5731 = t2;
  bool t3 = ov_valid_status_8b9994c4_c3c13714(v5731);
  bool t4 = (!t3);
  ovt_Response_dae56844* t5 = 0; (void)t5;
  if (t4) {
    ov_str* t6 = ov_str_new("bad status", 10);
    ovt_Response_dae56844* t7 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t6);
    t5 = t7;
  } else {
    bool t8 = ov_set_status_7fbb65d5_46ec26ca(p0, l5758_1, v5731);
    int64_t t9 = l5758_1->f_number;
    ovt_Response_dae56844* t10 = ov_respond_issue_61e7c0dc_9db72996(p0, l5758_0, t9);
    t5 = t10;
  }
  return t5;
}

ovt_Response_dae56844* ov_h_set_status_1e1a3ecd_347ed678(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3) {
  struct ovenv_5758* t11 = GC_MALLOC(sizeof(struct ovenv_5758));
  t11->p3 = p3;
  t11->p0 = p0;
  ov_closure* t12 = ov_closure_mk((void*)ovl_ov_h_set_status_1e1a3ecd_347ed678_5758, t11);
  ovt_Response_dae56844* t13 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t12);
  return t13;
}
