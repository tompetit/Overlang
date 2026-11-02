#include "ov_types.h"

ov_str* ov_r_hex2_5aa34ebe_56800c4d(int64_t p0, ov_str* p1);

ov_str* ov_r_hex2_5aa34ebe_56800c4d(int64_t p0, ov_str* p1) {
  int64_t t1 = ov_div(p0, INT64_C(16), "r_hex2", "5aa34ebed8418fffc777c8f718b564829af3b6b3f933953b25c49c693cd77643");
  int64_t t2 = ov_div(p0, INT64_C(16), "r_hex2", "5aa34ebed8418fffc777c8f718b564829af3b6b3f933953b25c49c693cd77643");
  int64_t t3 = ov_add(t2, INT64_C(1), "r_hex2", "5aa34ebed8418fffc777c8f718b564829af3b6b3f933953b25c49c693cd77643");
  ov_str* t4 = ov_str_substr(p1, t1, t3);
  int64_t t5 = ov_mod(p0, INT64_C(16), "r_hex2", "5aa34ebed8418fffc777c8f718b564829af3b6b3f933953b25c49c693cd77643");
  int64_t t6 = ov_mod(p0, INT64_C(16), "r_hex2", "5aa34ebed8418fffc777c8f718b564829af3b6b3f933953b25c49c693cd77643");
  int64_t t7 = ov_add(t6, INT64_C(1), "r_hex2", "5aa34ebed8418fffc777c8f718b564829af3b6b3f933953b25c49c693cd77643");
  ov_str* t8 = ov_str_substr(p1, t5, t7);
  ov_str* t9 = ov_str_concat(t4, t8);
  return t9;
}
