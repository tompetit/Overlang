#include "ov_types.h"

ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_r_input_7aab51f2_751f51d6(ov_str* p0, ov_str* p1, ov_str* p2);

ov_str* ov_r_input_7aab51f2_751f51d6(ov_str* p0, ov_str* p1, ov_str* p2) {
  ov_str* t1 = ov_str_new("<input name=\"", 13);
  ov_str* t2 = ov_str_concat(t1, p0);
  ov_str* t3 = ov_str_new("\" value=\"", 9);
  ov_str* t4 = ov_str_concat(t2, t3);
  ov_str* t5 = ov_html_escape_114cb911_d5a1eb27(p1);
  ov_str* t6 = ov_str_concat(t4, t5);
  ov_str* t7 = ov_str_new("\" placeholder=\"", 15);
  ov_str* t8 = ov_str_concat(t6, t7);
  ov_str* t9 = ov_str_concat(t8, p2);
  ov_str* t10 = ov_str_new("\">", 2);
  ov_str* t11 = ov_str_concat(t9, t10);
  return t11;
}
