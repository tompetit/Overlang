#include "ov_types.h"

int64_t ov_create_issue_c9275d15_8de03dd4(int64_t, int64_t, ov_str*, ov_str*, int64_t, int64_t, ov_list*);
int64_t ov_rt_priority_d125cc8e_7a48656b(ov_str*, int64_t);
ov_list* ov_parse_label_names_38f63ee7_6e2c6f9a(ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ov_str* ov_form_text_f01110b3_649a9e9b(ov_str*, ov_str*);
ovt_Option_7fe81862* ov_find_user_4bac60c8_5f9c3bd0(int64_t, ov_str*);
ovt_Option_c278faef* ov_find_label_589d5a55_3b09949d(int64_t, int64_t, ov_str*);
ovt_Response_dae56844* ov_resp_error_b47b9fc9_7be2787d(int64_t, ov_str*);
ovt_Response_dae56844* ov_respond_issue_61e7c0dc_9db72996(int64_t, ovt_Project_73b03799*, int64_t);
ovt_Response_dae56844* ov_rt_bad_priority_626b8671_0dd3d100(int64_t);
ovt_Response_dae56844* ov_rt_create_issue_a5cfd3d8_60cdb210(int64_t p0, ovt_Project_73b03799* p1, ov_str* p2);

ovt_Response_dae56844* ov_rt_create_issue_a5cfd3d8_60cdb210(int64_t p0, ovt_Project_73b03799* p1, ov_str* p2) {
  ov_str* t1 = ov_str_new("title", 5);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p2, t1);
  ov_str* v5387 = t2;
  ov_str* t3 = ov_str_new("", 0);
  bool t4 = ov_str_eq(v5387, t3);
  if (t4) {
    ov_str* t5 = ov_str_new("title required", 14);
    ovt_Response_dae56844* t6 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t5);
    return t6;
  }
  ov_str* t7 = ov_str_new("priority", 8);
  ov_str* t8 = ov_form_line_31e5aa84_d8f1245a(p2, t7);
  int64_t t9 = ov_rt_priority_d125cc8e_7a48656b(t8, INT64_C(3));
  int64_t v5400 = t9;
  bool t10 = (v5400 < INT64_C(1));
  bool t11 = t10;
  if (!t11) {
    bool t12 = (v5400 > INT64_C(4));
    t11 = t12;
  }
  if (t11) {
    ovt_Response_dae56844* t13 = ov_rt_bad_priority_626b8671_0dd3d100(v5400);
    return t13;
  }
  ov_str* t14 = ov_str_new("assignee", 8);
  ov_str* t15 = ov_form_line_31e5aa84_d8f1245a(p2, t14);
  ov_str* v5419 = t15;
  int64_t v5424 = INT64_C(0);
  (void)v5424;
  ov_str* t16 = ov_str_new("", 0);
  bool t17 = (!ov_str_eq(v5419, t16));
  if (t17) {
    ovt_Option_7fe81862* t18 = ov_find_user_4bac60c8_5f9c3bd0(p0, v5419);
    bool t19 = false; (void)t19;
    if (!t19) {
      if ((t18->tag == 0)) {
        ovt_User_9c127ab2* v5433 = t18->u.c0.f0;
        t19 = true;
        int64_t t20 = v5433->f_id;
        v5424 = t20;
      }
    }
    if (!t19) {
      if ((t18->tag == 1)) {
        t19 = true;
        ov_str* t21 = ov_str_new("no such user: ", 14);
        ov_str* t22 = ov_str_concat(t21, v5419);
        ovt_Response_dae56844* t23 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t22);
        return t23;
      }
    }
  }
  ov_list* t24 = ov_list_empty();
  ov_list* v5448 = t24;
  (void)v5448;
  ov_str* t25 = ov_str_new("labels", 6);
  ov_str* t26 = ov_form_line_31e5aa84_d8f1245a(p2, t25);
  ov_list* t27 = ov_parse_label_names_38f63ee7_6e2c6f9a(t26);
  ov_list* t28 = t27;
  for (int64_t t29 = 0; t29 < t28->len; t29++) {
    ov_str* t30 = ((ov_str*)(t28->items[t29]).p);
    ov_str* v5450 = t30;
    int64_t t32 = p1->f_id;
    ovt_Option_c278faef* t33 = ov_find_label_589d5a55_3b09949d(p0, t32, v5450);
    bool t34 = false; (void)t34;
    if (!t34) {
      if ((t33->tag == 0)) {
        ovt_Label_da07b2c4* v5463 = t33->u.c0.f0;
        t34 = true;
        ov_list* t35 = v5448;
        int64_t t36 = v5463->f_id;
        ov_list* t37 = ov_list_push(t35, (ov_value){.i = t36});
        v5448 = t37;
      }
    }
    if (!t34) {
      if ((t33->tag == 1)) {
        t34 = true;
        ov_str* t38 = ov_str_new("no such label: ", 15);
        ov_str* t39 = ov_str_concat(t38, v5450);
        ovt_Response_dae56844* t40 = ov_resp_error_b47b9fc9_7be2787d(INT64_C(422), t39);
        return t40;
      }
    }
  }
  int64_t t41 = p1->f_id;
  ov_str* t42 = ov_str_new("body", 4);
  ov_str* t43 = ov_form_text_f01110b3_649a9e9b(p2, t42);
  int64_t t44 = v5424;
  ov_list* t45 = v5448;
  ov_list* t46 = ovsup_unique(t45, OV_KEY_INT);
  int64_t t47 = ov_create_issue_c9275d15_8de03dd4(p0, t41, v5387, t43, v5400, t44, t46);
  int64_t v5480 = t47;
  ovt_Response_dae56844* t48 = ov_respond_issue_61e7c0dc_9db72996(p0, p1, v5480);
  return t48;
}
