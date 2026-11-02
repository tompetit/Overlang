#include "ov_types.h"

bool ov_is_none_4fb39622_28b94b57(ovt_Option_7fe81862*);
bool ov_set_assignee_df36439c_2f12287e(int64_t, ovt_Issue_b825796a*, ovt_Option_7fe81862*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Option_7fe81862* ov_find_user_4bac60c8_5f9c3bd0(int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t, ovt_Project_73b03799*, int64_t);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_assign_ec816036_c8b031b0(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3);

struct ovenv_5801 { ov_str* p3; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_assign_ec816036_c8b031b0_5801(void* ovenv, ovt_Project_73b03799* l5801_0, ovt_Issue_b825796a* l5801_1) {
  struct ovenv_5801* env = (struct ovenv_5801*)ovenv;
  ov_str* p3 = env->p3;
  int64_t p0 = env->p0;
  ov_str* t1 = ov_str_new("assignee", 8);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p3, t1);
  ov_str* v5764 = t2;
  ovt_Option_7fe81862* t3 = ov_find_user_4bac60c8_5f9c3bd0(p0, v5764);
  ovt_Option_7fe81862* v5769 = t3;
  ov_str* t4 = ov_str_new("", 0);
  bool t5 = (!ov_str_eq(v5764, t4));
  bool t6 = t5;
  if (t6) {
    bool t7 = ov_is_none_4fb39622_28b94b57(v5769);
    t6 = t7;
  }
  ovt_Response_dae56844* t8 = 0; (void)t8;
  if (t6) {
    ov_str* t9 = ov_str_new("no such user: ", 14);
    ov_str* t10 = ov_str_concat(t9, v5764);
    ovt_Response_dae56844* t11 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t10);
    t8 = t11;
  } else {
    bool t12 = ov_set_assignee_df36439c_2f12287e(p0, l5801_1, v5769);
    int64_t t13 = l5801_1->f_number;
    ovt_Response_dae56844* t14 = ov_respond_issue_61e7c0dc_9db72996(p0, l5801_0, t13);
    t8 = t14;
  }
  return t8;
}

ovt_Response_dae56844* ov_h_assign_ec816036_c8b031b0(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3) {
  struct ovenv_5801* t15 = GC_MALLOC(sizeof(struct ovenv_5801));
  t15->p3 = p3;
  t15->p0 = p0;
  ov_closure* t16 = ov_closure_mk((void*)ovl_ov_h_assign_ec816036_c8b031b0_5801, t15);
  ovt_Response_dae56844* t17 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t16);
  return t17;
}
