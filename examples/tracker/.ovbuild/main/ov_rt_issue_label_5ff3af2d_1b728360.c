#include "ov_types.h"

int64_t ov_add_label_fa06f62b_2620ec10(int64_t, ovt_Issue_b825796a*, ovt_Label_da07b2c4*);
int64_t ov_remove_label_32f284ac_49a29792(int64_t, ovt_Issue_b825796a*, ovt_Label_da07b2c4*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Option_c278faef* ov_find_label_589d5a55_3b09949d(int64_t, int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t, ovt_Project_73b03799*, int64_t);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_rt_issue_label_5ff3af2d_1b728360(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3, bool p4);

struct ovenv_5920 { ov_str* p3; int64_t p0; bool p4; };

static ovt_Response_dae56844* ovl_ov_rt_issue_label_5ff3af2d_1b728360_5920(void* ovenv, ovt_Project_73b03799* l5920_0, ovt_Issue_b825796a* l5920_1) {
  struct ovenv_5920* env = (struct ovenv_5920*)ovenv;
  ov_str* p3 = env->p3;
  int64_t p0 = env->p0;
  bool p4 = env->p4;
  ov_str* t1 = ov_str_new("label", 5);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p3, t1);
  ov_str* v5869 = t2;
  ov_str* t3 = ov_str_new("", 0);
  bool t4 = ov_str_eq(v5869, t3);
  ovt_Response_dae56844* t5 = 0; (void)t5;
  if (t4) {
    ov_str* t6 = ov_str_new("label required", 14);
    ovt_Response_dae56844* t7 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t6);
    t5 = t7;
  } else {
    int64_t t8 = l5920_0->f_id;
    ovt_Option_c278faef* t9 = ov_find_label_589d5a55_3b09949d(p0, t8, v5869);
    ovt_Response_dae56844* t10 = 0; (void)t10;
    bool t11 = false; (void)t11;
    if (!t11) {
      if ((t9->tag == 1)) {
        t11 = true;
        ov_str* t12 = ov_str_new("no such label: ", 15);
        ov_str* t13 = ov_str_concat(t12, v5869);
        ovt_Response_dae56844* t14 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t13);
        t10 = t14;
      }
    }
    if (!t11) {
      if ((t9->tag == 0)) {
        ovt_Label_da07b2c4* v5894 = t9->u.c0.f0;
        t11 = true;
        if (p4) {
          int64_t t15 = ov_add_label_fa06f62b_2620ec10(p0, l5920_1, v5894);
        } else {
          int64_t t16 = ov_remove_label_32f284ac_49a29792(p0, l5920_1, v5894);
        }
        int64_t t17 = l5920_1->f_number;
        ovt_Response_dae56844* t18 = ov_respond_issue_61e7c0dc_9db72996(p0, l5920_0, t17);
        t10 = t18;
      }
    }
    t5 = t10;
  }
  return t5;
}

ovt_Response_dae56844* ov_rt_issue_label_5ff3af2d_1b728360(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3, bool p4) {
  struct ovenv_5920* t19 = GC_MALLOC(sizeof(struct ovenv_5920));
  t19->p3 = p3;
  t19->p0 = p0;
  t19->p4 = p4;
  ov_closure* t20 = ov_closure_mk((void*)ovl_ov_rt_issue_label_5ff3af2d_1b728360_5920, t19);
  ovt_Response_dae56844* t21 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t20);
  return t21;
}
