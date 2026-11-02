#include "ov_types.h"

bool ov_all_ba8a189d_8d783bfe(ov_list*, ov_closure*);
bool ov_is_digit_d7695ac0_63fa906b(ov_str*);
bool ov_is_upper_474465f2_876a8502(ov_str*);
bool ov_valid_project_key_0acaa387_69ec39bf(ov_str* p0);

static bool ovl_ov_valid_project_key_0acaa387_69ec39bf_1431(void* ovenv, ov_str* l1431_0) {
  bool t9 = ov_is_upper_474465f2_876a8502(l1431_0);
  bool t10 = t9;
  if (!t10) {
    bool t11 = ov_is_digit_d7695ac0_63fa906b(l1431_0);
    t10 = t11;
  }
  return t10;
}

bool ov_valid_project_key_0acaa387_69ec39bf(ov_str* p0) {
  int64_t t1 = ov_str_len_cps(p0);
  int64_t v1405 = t1;
  bool t2 = (v1405 >= INT64_C(2));
  bool t3 = t2;
  if (t3) {
    bool t4 = (v1405 <= INT64_C(10));
    t3 = t4;
  }
  bool t5 = t3;
  if (t5) {
    bool t6 = ov_is_upper_474465f2_876a8502(p0);
    t5 = t6;
  }
  bool t7 = t5;
  if (t7) {
    ov_list* t8 = ov_str_chars(p0);
    ov_closure* t12 = ov_closure_mk((void*)ovl_ov_valid_project_key_0acaa387_69ec39bf_1431, NULL);
    bool t13 = ov_all_ba8a189d_8d783bfe(t8, t12);
    t7 = t13;
  }
  return t7;
}
