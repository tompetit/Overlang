#include "ov_types.h"

bool ov_is_alnum_6cf22f95_00c25202(ov_str*);
int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
ov_list* ov_map_9a617375_9fd4b57a(ov_list*, ov_closure*);
ov_list* ov_r_utf8_701fa1b2_27a19eb8(int64_t);
ov_str* ov_r_hex2_5aa34ebe_56800c4d(int64_t, ov_str*);
ov_str* ov_url_encode_9b07eb27_7c143021(ov_str* p0);

static ov_str* ovl_ov_url_encode_9b07eb27_7c143021_3180(void* ovenv, int64_t l3180_0) {
  ov_str* t28 = ov_str_new("%", 1);
  ov_str* t29 = ov_str_new("0123456789ABCDEF", 16);
  ov_str* t30 = ov_r_hex2_5aa34ebe_56800c4d(l3180_0, t29);
  ov_str* t31 = ov_str_concat(t28, t30);
  return t31;
}

ov_str* ov_url_encode_9b07eb27_7c143021(ov_str* p0) {
  ov_str* t1 = ov_str_new("", 0);
  ov_str* v3133 = t1;
  (void)v3133;
  ov_list* t2 = ov_str_chars(p0);
  ov_list* t3 = t2;
  for (int64_t t4 = 0; t4 < t3->len; t4++) {
    ov_str* t5 = ((ov_str*)(t3->items[t4]).p);
    ov_str* v3135 = t5;
    bool t7 = ov_is_alnum_6cf22f95_00c25202(v3135);
    bool t8 = t7;
    if (!t8) {
      ov_str* t9 = ov_str_new("-", 1);
      bool t10 = ov_str_eq(v3135, t9);
      t8 = t10;
    }
    bool t11 = t8;
    if (!t11) {
      ov_str* t12 = ov_str_new("_", 1);
      bool t13 = ov_str_eq(v3135, t12);
      t11 = t13;
    }
    bool t14 = t11;
    if (!t14) {
      ov_str* t15 = ov_str_new(".", 1);
      bool t16 = ov_str_eq(v3135, t15);
      t14 = t16;
    }
    bool t17 = t14;
    if (!t17) {
      ov_str* t18 = ov_str_new("~", 1);
      bool t19 = ov_str_eq(v3135, t18);
      t17 = t19;
    }
    if (t17) {
      ov_str* t20 = v3133;
      ov_str* t21 = ov_str_concat(t20, v3135);
      v3133 = t21;
    } else {
      bool t22 = false;
      int64_t t23 = ov_str_code_point(v3135, &t22);
      ovt_Option_d5f2c30a* t24 = ((t22) ? ovt_Option_d5f2c30a_mk0(t23) : ovt_Option_d5f2c30a_mk1());
      int64_t t25 = ov_unwrap_or_e7341bb9_bde9d67c(t24, INT64_C(0));
      int64_t v3161 = t25;
      ov_str* t26 = v3133;
      ov_list* t27 = ov_r_utf8_701fa1b2_27a19eb8(v3161);
      ov_closure* t32 = ov_closure_mk((void*)ovl_ov_url_encode_9b07eb27_7c143021_3180, NULL);
      ov_list* t33 = ov_map_9a617375_9fd4b57a(t27, t32);
      ov_str* t34 = ov_str_new("", 0);
      ov_str* t35 = ov_str_join(t33, t34);
      ov_str* t36 = ov_str_concat(t26, t35);
      v3133 = t36;
    }
  }
  ov_str* t37 = v3133;
  return t37;
}
