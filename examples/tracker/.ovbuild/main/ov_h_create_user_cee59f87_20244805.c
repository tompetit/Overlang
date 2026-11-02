#include "ov_types.h"

bool ov_is_some_5d16aee5_289a7f35(ovt_Option_7fe81862*);
bool ov_valid_user_name_eb496c11_934af82b(ov_str*);
int64_t ov_create_user_f316dd39_339cb8a1(int64_t, ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Option_7fe81862* ov_find_user_4bac60c8_5f9c3bd0(int64_t, ov_str*);
ovt_Response_dae56844* ov_h_users_page_4d7202e0_54da708a(int64_t);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_h_create_user_cee59f87_20244805(int64_t p0, ov_str* p1);

ovt_Response_dae56844* ov_h_create_user_cee59f87_20244805(int64_t p0, ov_str* p1) {
  ov_str* t1 = ov_str_new("name", 4);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p1, t1);
  ov_str* v5212 = t2;
  ov_str* t3 = ov_str_new("", 0);
  bool t4 = ov_str_eq(v5212, t3);
  ovt_Response_dae56844* t5 = 0; (void)t5;
  if (t4) {
    ov_str* t6 = ov_str_new("name required", 13);
    ovt_Response_dae56844* t7 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t6);
    t5 = t7;
  } else {
    bool t8 = ov_valid_user_name_eb496c11_934af82b(v5212);
    bool t9 = (!t8);
    ovt_Response_dae56844* t10 = 0; (void)t10;
    if (t9) {
      ov_str* t11 = ov_str_new("bad name", 8);
      ovt_Response_dae56844* t12 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t11);
      t10 = t12;
    } else {
      ovt_Option_7fe81862* t13 = ov_find_user_4bac60c8_5f9c3bd0(p0, v5212);
      bool t14 = ov_is_some_5d16aee5_289a7f35(t13);
      ovt_Response_dae56844* t15 = 0; (void)t15;
      if (t14) {
        ov_str* t16 = ov_str_new("user exists", 11);
        ovt_Response_dae56844* t17 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t16);
        t15 = t17;
      } else {
        int64_t t18 = ov_create_user_f316dd39_339cb8a1(p0, v5212);
        ovt_Response_dae56844* t19 = ov_h_users_page_4d7202e0_54da708a(p0);
        t15 = t19;
      }
      t10 = t15;
    }
    t5 = t10;
  }
  return t5;
}
