#include "ov_types.h"

bool ov_all_ba8a189d_8d783bfe(ov_list*, ov_closure*);
bool ov_is_digit_d7695ac0_63fa906b(ov_str*);
bool ov_is_lower_390d9774_f60e8723(ov_str*);
bool ov_valid_label_name_abbf85dd_9802eab5(ov_str* p0);

static bool ovl_ov_valid_label_name_abbf85dd_9802eab5_1460(void* ovenv, ov_str* l1460_0) {
  bool t7 = ov_is_lower_390d9774_f60e8723(l1460_0);
  bool t8 = t7;
  if (!t8) {
    bool t9 = ov_is_digit_d7695ac0_63fa906b(l1460_0);
    t8 = t9;
  }
  bool t10 = t8;
  if (!t10) {
    ov_str* t11 = ov_str_new("-", 1);
    bool t12 = ov_str_eq(l1460_0, t11);
    t10 = t12;
  }
  return t10;
}

bool ov_valid_label_name_abbf85dd_9802eab5(ov_str* p0) {
  int64_t t1 = ov_str_len_cps(p0);
  int64_t v1434 = t1;
  bool t2 = (v1434 >= INT64_C(1));
  bool t3 = t2;
  if (t3) {
    bool t4 = (v1434 <= INT64_C(20));
    t3 = t4;
  }
  bool t5 = t3;
  if (t5) {
    ov_list* t6 = ov_str_chars(p0);
    ov_closure* t13 = ov_closure_mk((void*)ovl_ov_valid_label_name_abbf85dd_9802eab5_1460, NULL);
    bool t14 = ov_all_ba8a189d_8d783bfe(t6, t13);
    t5 = t14;
  }
  return t5;
}
