#include "ov_types.h"

bool ov_all_ba8a189d_8d783bfe(ov_list*, ov_closure*);
bool ov_is_alnum_6cf22f95_00c25202(ov_str*);
bool ov_valid_user_name_eb496c11_934af82b(ov_str* p0);

static bool ovl_ov_valid_user_name_eb496c11_934af82b_1402(void* ovenv, ov_str* l1402_0) {
  bool t7 = ov_is_alnum_6cf22f95_00c25202(l1402_0);
  bool t8 = t7;
  if (!t8) {
    ov_str* t9 = ov_str_new("_", 1);
    bool t10 = ov_str_eq(l1402_0, t9);
    t8 = t10;
  }
  bool t11 = t8;
  if (!t11) {
    ov_str* t12 = ov_str_new("-", 1);
    bool t13 = ov_str_eq(l1402_0, t12);
    t11 = t13;
  }
  bool t14 = t11;
  if (!t14) {
    ov_str* t15 = ov_str_new(".", 1);
    bool t16 = ov_str_eq(l1402_0, t15);
    t14 = t16;
  }
  return t14;
}

bool ov_valid_user_name_eb496c11_934af82b(ov_str* p0) {
  int64_t t1 = ov_str_len_cps(p0);
  int64_t v1372 = t1;
  bool t2 = (v1372 >= INT64_C(1));
  bool t3 = t2;
  if (t3) {
    bool t4 = (v1372 <= INT64_C(32));
    t3 = t4;
  }
  bool t5 = t3;
  if (t5) {
    ov_list* t6 = ov_str_chars(p0);
    ov_closure* t17 = ov_closure_mk((void*)ovl_ov_valid_user_name_eb496c11_934af82b_1402, NULL);
    bool t18 = ov_all_ba8a189d_8d783bfe(t6, t17);
    t5 = t18;
  }
  return t5;
}
