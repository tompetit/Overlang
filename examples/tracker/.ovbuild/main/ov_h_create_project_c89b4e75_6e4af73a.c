#include "ov_types.h"

bool ov_is_some_5d16aee5_ca843493(ovt_Option_da3cc0d5*);
bool ov_valid_project_key_0acaa387_69ec39bf(ov_str*);
int64_t ov_create_project_2373b13f_51d6e4b6(int64_t, ov_str*, ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Filter_50e09095* ov_no_filter_53ccd73e_859849c3(void);
ovt_Option_da3cc0d5* ov_find_project_da7b7099_8f536708(int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_project_dbff6379_4e076edf(int64_t, ovt_Project_73b03799*, ovt_Filter_50e09095*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Response_dae56844* ov_h_create_project_c89b4e75_6e4af73a(int64_t p0, ov_str* p1);

struct ovenv_5316 { int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_create_project_c89b4e75_6e4af73a_5316(void* ovenv, ovt_Project_73b03799* l5316_0) {
  struct ovenv_5316* env = (struct ovenv_5316*)ovenv;
  int64_t p0 = env->p0;
  ovt_Filter_50e09095* t26 = ov_no_filter_53ccd73e_859849c3();
  ovt_Response_dae56844* t27 = ov_respond_project_dbff6379_4e076edf(p0, l5316_0, t26);
  return t27;
}

ovt_Response_dae56844* ov_h_create_project_c89b4e75_6e4af73a(int64_t p0, ov_str* p1) {
  ov_str* t1 = ov_str_new("key", 3);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p1, t1);
  ov_str* v5260 = t2;
  ov_str* t3 = ov_str_new("name", 4);
  ov_str* t4 = ov_form_line_31e5aa84_d8f1245a(p1, t3);
  ov_str* v5265 = t4;
  ov_str* t5 = ov_str_new("", 0);
  bool t6 = ov_str_eq(v5260, t5);
  ovt_Response_dae56844* t7 = 0; (void)t7;
  if (t6) {
    ov_str* t8 = ov_str_new("key required", 12);
    ovt_Response_dae56844* t9 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t8);
    t7 = t9;
  } else {
    bool t10 = ov_valid_project_key_0acaa387_69ec39bf(v5260);
    bool t11 = (!t10);
    ovt_Response_dae56844* t12 = 0; (void)t12;
    if (t11) {
      ov_str* t13 = ov_str_new("bad key", 7);
      ovt_Response_dae56844* t14 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t13);
      t12 = t14;
    } else {
      ov_str* t15 = ov_str_new("", 0);
      bool t16 = ov_str_eq(v5265, t15);
      ovt_Response_dae56844* t17 = 0; (void)t17;
      if (t16) {
        ov_str* t18 = ov_str_new("name required", 13);
        ovt_Response_dae56844* t19 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t18);
        t17 = t19;
      } else {
        ovt_Option_da3cc0d5* t20 = ov_find_project_da7b7099_8f536708(p0, v5260);
        bool t21 = ov_is_some_5d16aee5_ca843493(t20);
        ovt_Response_dae56844* t22 = 0; (void)t22;
        if (t21) {
          ov_str* t23 = ov_str_new("project exists", 14);
          ovt_Response_dae56844* t24 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t23);
          t22 = t24;
        } else {
          int64_t t25 = ov_create_project_2373b13f_51d6e4b6(p0, v5260, v5265);
          struct ovenv_5316* t28 = GC_MALLOC(sizeof(struct ovenv_5316));
          t28->p0 = p0;
          ov_closure* t29 = ov_closure_mk((void*)ovl_ov_h_create_project_c89b4e75_6e4af73a_5316, t28);
          ovt_Response_dae56844* t30 = ov_with_project_4b2a8feb_720ea08b(p0, v5260, t29);
          t22 = t30;
        }
        t17 = t22;
      }
      t12 = t17;
    }
    t7 = t12;
  }
  return t7;
}
