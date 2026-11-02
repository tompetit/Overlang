#include "ov_types.h"

int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
ov_str* ov_r_hex2_5aa34ebe_56800c4d(int64_t, ov_str*);
ov_str* ov_json_str_a498952b_d002ebb1(ov_str* p0);

ov_str* ov_json_str_a498952b_d002ebb1(ov_str* p0) {
  ov_str* t1 = ov_str_new("\"", 1);
  ov_str* v3188 = t1;
  (void)v3188;
  ov_list* t2 = ov_str_chars(p0);
  ov_list* t3 = t2;
  for (int64_t t4 = 0; t4 < t3->len; t4++) {
    ov_str* t5 = ((ov_str*)(t3->items[t4]).p);
    ov_str* v3190 = t5;
    bool t7 = false;
    int64_t t8 = ov_str_code_point(v3190, &t7);
    ovt_Option_d5f2c30a* t9 = ((t7) ? ovt_Option_d5f2c30a_mk0(t8) : ovt_Option_d5f2c30a_mk1());
    int64_t t10 = ov_unwrap_or_e7341bb9_bde9d67c(t9, INT64_C(0));
    int64_t v3194 = t10;
    ov_str* t11 = v3188;
    ov_str* t12 = ov_str_new("\\", 1);
    bool t13 = ov_str_eq(v3190, t12);
    ov_str* t14 = 0; (void)t14;
    if (t13) {
      ov_str* t15 = ov_str_new("\\\\", 2);
      t14 = t15;
    } else {
      ov_str* t16 = ov_str_new("\"", 1);
      bool t17 = ov_str_eq(v3190, t16);
      ov_str* t18 = 0; (void)t18;
      if (t17) {
        ov_str* t19 = ov_str_new("\\\"", 2);
        t18 = t19;
      } else {
        ov_str* t20 = ov_str_new("\n", 1);
        bool t21 = ov_str_eq(v3190, t20);
        ov_str* t22 = 0; (void)t22;
        if (t21) {
          ov_str* t23 = ov_str_new("\\n", 2);
          t22 = t23;
        } else {
          ov_str* t24 = ov_str_new("\015", 1);
          bool t25 = ov_str_eq(v3190, t24);
          ov_str* t26 = 0; (void)t26;
          if (t25) {
            ov_str* t27 = ov_str_new("\\r", 2);
            t26 = t27;
          } else {
            ov_str* t28 = ov_str_new("\t", 1);
            bool t29 = ov_str_eq(v3190, t28);
            ov_str* t30 = 0; (void)t30;
            if (t29) {
              ov_str* t31 = ov_str_new("\\t", 2);
              t30 = t31;
            } else {
              bool t32 = (v3194 < INT64_C(32));
              ov_str* t33 = 0; (void)t33;
              if (t32) {
                ov_str* t34 = ov_str_new("\\u00", 4);
                ov_str* t35 = ov_str_new("0123456789abcdef", 16);
                ov_str* t36 = ov_r_hex2_5aa34ebe_56800c4d(v3194, t35);
                ov_str* t37 = ov_str_concat(t34, t36);
                t33 = t37;
              } else {
                t33 = v3190;
              }
              t30 = t33;
            }
            t26 = t30;
          }
          t22 = t26;
        }
        t18 = t22;
      }
      t14 = t18;
    }
    ov_str* t38 = ov_str_concat(t11, t14);
    v3188 = t38;
  }
  ov_str* t39 = v3188;
  ov_str* t40 = ov_str_new("\"", 1);
  ov_str* t41 = ov_str_concat(t39, t40);
  return t41;
}
