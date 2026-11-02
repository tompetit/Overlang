#include "ov_types.h"

bool ov_is_some_5d16aee5_0ba6f2de(ovt_Option_c278faef*);
bool ov_valid_label_name_abbf85dd_9802eab5(ov_str*);
int64_t ov_create_label_da024719_33b12a1b(int64_t, int64_t, ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Filter_50e09095* ov_no_filter_53ccd73e_859849c3(void);
ovt_Option_c278faef* ov_find_label_589d5a55_3b09949d(int64_t, int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_project_dbff6379_4e076edf(int64_t, ovt_Project_73b03799*, ovt_Filter_50e09095*);
ovt_Response_dae56844* ov_with_project_4b2a8feb_720ea08b(int64_t, ov_str*, ov_closure*);
ovt_Response_dae56844* ov_h_create_label_2c50ed71_19fb2a59(int64_t p0, ov_str* p1, ov_str* p2);

struct ovenv_5577 { ov_str* p2; int64_t p0; };

static ovt_Response_dae56844* ovl_ov_h_create_label_2c50ed71_19fb2a59_5577(void* ovenv, ovt_Project_73b03799* l5577_0) {
  struct ovenv_5577* env = (struct ovenv_5577*)ovenv;
  ov_str* p2 = env->p2;
  int64_t p0 = env->p0;
  ov_str* t1 = ov_str_new("name", 4);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p2, t1);
  ov_str* v5528 = t2;
  ov_str* t3 = ov_str_new("", 0);
  bool t4 = ov_str_eq(v5528, t3);
  ovt_Response_dae56844* t5 = 0; (void)t5;
  if (t4) {
    ov_str* t6 = ov_str_new("name required", 13);
    ovt_Response_dae56844* t7 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t6);
    t5 = t7;
  } else {
    bool t8 = ov_valid_label_name_abbf85dd_9802eab5(v5528);
    bool t9 = (!t8);
    ovt_Response_dae56844* t10 = 0; (void)t10;
    if (t9) {
      ov_str* t11 = ov_str_new("bad label", 9);
      ovt_Response_dae56844* t12 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t11);
      t10 = t12;
    } else {
      int64_t t13 = l5577_0->f_id;
      ovt_Option_c278faef* t14 = ov_find_label_589d5a55_3b09949d(p0, t13, v5528);
      bool t15 = ov_is_some_5d16aee5_0ba6f2de(t14);
      ovt_Response_dae56844* t16 = 0; (void)t16;
      if (t15) {
        ov_str* t17 = ov_str_new("label exists", 12);
        ovt_Response_dae56844* t18 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t17);
        t16 = t18;
      } else {
        int64_t t19 = l5577_0->f_id;
        int64_t t20 = ov_create_label_da024719_33b12a1b(p0, t19, v5528);
        ovt_Filter_50e09095* t21 = ov_no_filter_53ccd73e_859849c3();
        ovt_Response_dae56844* t22 = ov_respond_project_dbff6379_4e076edf(p0, l5577_0, t21);
        t16 = t22;
      }
      t10 = t16;
    }
    t5 = t10;
  }
  return t5;
}

ovt_Response_dae56844* ov_h_create_label_2c50ed71_19fb2a59(int64_t p0, ov_str* p1, ov_str* p2) {
  struct ovenv_5577* t23 = GC_MALLOC(sizeof(struct ovenv_5577));
  t23->p2 = p2;
  t23->p0 = p0;
  ov_closure* t24 = ov_closure_mk((void*)ovl_ov_h_create_label_2c50ed71_19fb2a59_5577, t23);
  ovt_Response_dae56844* t25 = ov_with_project_4b2a8feb_720ea08b(p0, p1, t24);
  return t25;
}
