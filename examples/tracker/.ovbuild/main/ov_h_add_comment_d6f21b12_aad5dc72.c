#include "ov_types.h"

int64_t ov_add_comment_cccdec46_bf9a060b(int64_t, ovt_Issue_b825796a*, ovt_User_9c127ab2*, ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ov_str* ov_form_text_f01110b3_649a9e9b(ov_str*, ov_str*);
ovt_Option_7fe81862* ov_find_user_4bac60c8_5f9c3bd0(int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t, ovt_Project_73b03799*, int64_t);
ovt_Response_dae56844* ov_with_issue_c1a88074_8cf0ad07(int64_t, ov_str*, int64_t, ov_closure*);
ovt_Response_dae56844* ov_h_add_comment_d6f21b12_aad5dc72(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3);

struct ovenv_5863 { ov_str* p3; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_add_comment_d6f21b12_aad5dc72_5863(void* ovenv, ovt_Project_73b03799* l5863_0, ovt_Issue_b825796a* l5863_1) {
  struct ovenv_5863* env = (struct ovenv_5863*)ovenv;
  ov_str* p3 = env->p3;
  int64_t p0 = env->p0;
  ov_str* t1 = ov_str_new("author", 6);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p3, t1);
  ov_str* v5807 = t2;
  ov_str* t3 = ov_str_new("body", 4);
  ov_str* t4 = ov_form_text_f01110b3_649a9e9b(p3, t3);
  ov_str* v5812 = t4;
  ov_str* t5 = ov_str_new("", 0);
  bool t6 = ov_str_eq(v5807, t5);
  ovt_Response_dae56844* t7 = 0; (void)t7;
  if (t6) {
    ov_str* t8 = ov_str_new("author required", 15);
    ovt_Response_dae56844* t9 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t8);
    t7 = t9;
  } else {
    ovt_Option_7fe81862* t10 = ov_find_user_4bac60c8_5f9c3bd0(p0, v5807);
    ovt_Response_dae56844* t11 = 0; (void)t11;
    bool t12 = false; (void)t12;
    if (!t12) {
      if ((t10->tag == 1)) {
        t12 = true;
        ov_str* t13 = ov_str_new("no such user: ", 14);
        ov_str* t14 = ov_str_concat(t13, v5807);
        ovt_Response_dae56844* t15 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t14);
        t11 = t15;
      }
    }
    if (!t12) {
      if ((t10->tag == 0)) {
        ov_str* t16 = ov_str_new("", 0);
        bool t17 = ov_str_eq(v5812, t16);
        if (t17) {
          t12 = true;
          ov_str* t18 = ov_str_new("body required", 13);
          ovt_Response_dae56844* t19 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t18);
          t11 = t19;
        }
      }
    }
    if (!t12) {
      if ((t10->tag == 0)) {
        ovt_User_9c127ab2* v5844 = t10->u.c0.f0;
        t12 = true;
        int64_t t20 = ov_add_comment_cccdec46_bf9a060b(p0, l5863_1, v5844, v5812);
        int64_t t21 = l5863_1->f_number;
        ovt_Response_dae56844* t22 = ov_respond_issue_61e7c0dc_9db72996(p0, l5863_0, t21);
        t11 = t22;
      }
    }
    t7 = t11;
  }
  return t7;
}

ovt_Response_dae56844* ov_h_add_comment_d6f21b12_aad5dc72(int64_t p0, ov_str* p1, int64_t p2, ov_str* p3) {
  struct ovenv_5863* t23 = GC_MALLOC(sizeof(struct ovenv_5863));
  t23->p3 = p3;
  t23->p0 = p0;
  ov_closure* t24 = ov_closure_mk((void*)ovl_ov_h_add_comment_d6f21b12_aad5dc72_5863, t23);
  ovt_Response_dae56844* t25 = ov_with_issue_c1a88074_8cf0ad07(p0, p1, p2, t24);
  return t25;
}
