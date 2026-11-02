#include "ov_types.h"

int64_t ov_edit_issue_e4d5981f_f522a832(int64_t, ovt_Issue_b825796a*, ov_str*, ov_str*, int64_t);
int64_t ov_rt_priority_d125cc8e_7a48656b(ov_str*, int64_t);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ov_str* ov_form_text_f01110b3_649a9e9b(ov_str*, ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t, ovt_Project_73b03799*, int64_t);
ovt_Response_dae56844* ov_rt_bad_priority_626b8671_0dd3d100(int64_t);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_edit_issue_58bc3aca_46ced106(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3);

struct ovenv_5725 { ov_str* p3; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_edit_issue_58bc3aca_46ced106_5725(void* ovenv, ovt_Project_73b03799* l5725_0, ovt_Issue_b825796a* l5725_1) {
  struct ovenv_5725* env = (struct ovenv_5725*)ovenv;
  ov_str* p3 = env->p3;
  int64_t p0 = env->p0;
  ov_str* t1 = ov_str_new("priority", 8);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p3, t1);
  int64_t t3 = l5725_1->f_priority;
  int64_t t4 = ov_rt_priority_d125cc8e_7a48656b(t2, t3);
  int64_t v5666 = t4;
  bool t5 = (v5666 < INT64_C(1));
  bool t6 = t5;
  if (!t6) {
    bool t7 = (v5666 > INT64_C(4));
    t6 = t7;
  }
  ovt_Response_dae56844* t8 = 0; (void)t8;
  if (t6) {
    ovt_Response_dae56844* t9 = ov_rt_bad_priority_626b8671_0dd3d100(v5666);
    t8 = t9;
  } else {
    ov_str* t10 = ov_str_new("title", 5);
    ov_str* t11 = ov_form_line_31e5aa84_d8f1245a(p3, t10);
    ov_str* v5685 = t11;
    ov_str* t12 = ov_str_new("body", 4);
    ov_str* t13 = ov_form_text_f01110b3_649a9e9b(p3, t12);
    ov_str* v5690 = t13;
    ov_str* t14 = ov_str_new("", 0);
    bool t15 = ov_str_eq(v5685, t14);
    ov_str* t16 = 0; (void)t16;
    if (t15) {
      ov_str* t17 = l5725_1->f_title;
      t16 = t17;
    } else {
      t16 = v5685;
    }
    ov_str* t18 = ov_str_new("", 0);
    bool t19 = ov_str_eq(v5690, t18);
    ov_str* t20 = 0; (void)t20;
    if (t19) {
      ov_str* t21 = l5725_1->f_body;
      t20 = t21;
    } else {
      t20 = v5690;
    }
    int64_t t22 = ov_edit_issue_e4d5981f_f522a832(p0, l5725_1, t16, t20, v5666);
    int64_t t23 = l5725_1->f_number;
    ovt_Response_dae56844* t24 = ov_respond_issue_61e7c0dc_9db72996(p0, l5725_0, t23);
    t8 = t24;
  }
  return t8;
}

ovt_Response_dae56844* ov_h_edit_issue_58bc3aca_46ced106(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3) {
  struct ovenv_5725* t25 = GC_MALLOC(sizeof(struct ovenv_5725));
  t25->p3 = p3;
  t25->p0 = p0;
  ov_closure* t26 = ov_closure_mk((void*)ovl_ov_h_edit_issue_58bc3aca_46ced106_5725, t25);
  ovt_Response_dae56844* t27 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t26);
  return t27;
}
