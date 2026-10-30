#include "ov_types.h"

ov_str* ov_id_form_f3edd237_f317a921(ov_str* p0, int64_t p1, ov_str* p2);

ov_str* ov_id_form_f3edd237_f317a921(ov_str* p0, int64_t p1, ov_str* p2) {
  ov_str* t1 = ov_str_new("<form method=\"post\" action=\"", 28);
  ov_str* t2 = ov_str_concat(t1, p0);
  ov_str* t3 = ov_str_new("\" style=\"display:inline\">", 25);
  ov_str* t4 = ov_str_concat(t2, t3);
  ov_str* t5 = ov_str_new("<input type=\"hidden\" name=\"id\" value=\"", 38);
  ov_str* t6 = ov_str_concat(t4, t5);
  ov_str* t7 = ov_show_int(p1);
  ov_str* t8 = ov_str_concat(t6, t7);
  ov_str* t9 = ov_str_new("\">", 2);
  ov_str* t10 = ov_str_concat(t8, t9);
  ov_str* t11 = ov_str_new("<button>", 8);
  ov_str* t12 = ov_str_concat(t10, t11);
  ov_str* t13 = ov_str_concat(t12, p2);
  ov_str* t14 = ov_str_new("</button></form>", 16);
  ov_str* t15 = ov_str_concat(t13, t14);
  return t15;
}
