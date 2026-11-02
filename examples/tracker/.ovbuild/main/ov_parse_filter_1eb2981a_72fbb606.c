#include "ov_types.h"

bool ov_valid_status_8b9994c4_c3c13714(ov_str*);
int64_t ov_parse_nat_1580a6df_b91b8eff(ov_str*);
ov_str* ov_form_line_31e5aa84_d8f1245a(ov_str*, ov_str*);
ovt_Result_3d1782a4* ov_parse_filter_1eb2981a_72fbb606(ov_str* p0);

ovt_Result_3d1782a4* ov_parse_filter_1eb2981a_72fbb606(ov_str* p0) {
  ov_str* t1 = ov_str_new("status", 6);
  ov_str* t2 = ov_form_line_31e5aa84_d8f1245a(p0, t1);
  ov_str* v4960 = t2;
  ov_str* t3 = ov_str_new("", 0);
  bool t4 = (!ov_str_eq(v4960, t3));
  bool t5 = t4;
  if (t5) {
    bool t6 = ov_valid_status_8b9994c4_c3c13714(v4960);
    bool t7 = (!t6);
    t5 = t7;
  }
  if (t5) {
    ov_str* t8 = ov_str_new("bad status", 10);
    ovt_Result_3d1782a4* t9 = ovt_Result_3d1782a4_mk1(t8);
    return t9;
  }
  ov_str* t10 = ov_str_new("priority", 8);
  ov_str* t11 = ov_form_line_31e5aa84_d8f1245a(p0, t10);
  ov_str* v4976 = t11;
  ov_str* t12 = ov_str_new("", 0);
  bool t13 = ov_str_eq(v4976, t12);
  int64_t t14 = 0; (void)t14;
  if (t13) {
    t14 = INT64_C(0);
  } else {
    int64_t t15 = ov_parse_nat_1580a6df_b91b8eff(v4976);
    t14 = t15;
  }
  int64_t v4981 = t14;
  ov_str* t16 = ov_str_new("", 0);
  bool t17 = (!ov_str_eq(v4976, t16));
  bool t18 = t17;
  if (t18) {
    bool t19 = (v4981 < INT64_C(1));
    bool t20 = t19;
    if (!t20) {
      bool t21 = (v4981 > INT64_C(4));
      t20 = t21;
    }
    t18 = t20;
  }
  if (t18) {
    ov_str* t22 = ov_str_new("bad priority", 12);
    ovt_Result_3d1782a4* t23 = ovt_Result_3d1782a4_mk1(t22);
    return t23;
  }
  ov_str* t24 = ov_str_new("page", 4);
  ov_str* t25 = ov_form_line_31e5aa84_d8f1245a(p0, t24);
  ov_str* v5005 = t25;
  ov_str* t26 = ov_str_new("", 0);
  bool t27 = ov_str_eq(v5005, t26);
  int64_t t28 = 0; (void)t28;
  if (t27) {
    t28 = INT64_C(1);
  } else {
    int64_t t29 = ov_parse_nat_1580a6df_b91b8eff(v5005);
    t28 = t29;
  }
  int64_t v5010 = t28;
  bool t30 = (v5010 < INT64_C(1));
  bool t31 = t30;
  if (!t31) {
    bool t32 = (v5010 > INT64_C(1000000));
    t31 = t32;
  }
  if (t31) {
    ov_str* t33 = ov_str_new("bad page", 8);
    ovt_Result_3d1782a4* t34 = ovt_Result_3d1782a4_mk1(t33);
    return t34;
  }
  ov_str* t35 = ov_str_new("assignee", 8);
  ov_str* t36 = ov_form_line_31e5aa84_d8f1245a(p0, t35);
  ov_str* t37 = ov_str_new("label", 5);
  ov_str* t38 = ov_form_line_31e5aa84_d8f1245a(p0, t37);
  ov_str* t39 = ov_str_new("q", 1);
  ov_str* t40 = ov_form_line_31e5aa84_d8f1245a(p0, t39);
  ovt_Filter_50e09095* t41 = ovt_Filter_50e09095_mk(v4960, t36, t38, v4981, t40, v5010);
  ovt_Result_3d1782a4* t42 = ovt_Result_3d1782a4_mk0(t41);
  return t42;
}
