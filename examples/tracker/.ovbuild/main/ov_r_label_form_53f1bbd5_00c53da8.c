#include "ov_types.h"

ov_str* ov_r_label_form_53f1bbd5_00c53da8(ov_str* p0, ov_str* p1, ov_str* p2);

ov_str* ov_r_label_form_53f1bbd5_00c53da8(ov_str* p0, ov_str* p1, ov_str* p2) {
  ov_str* t1 = ov_str_new("<form method=\"post\" action=\"", 28);
  ov_str* t2 = ov_str_concat(t1, p0);
  ov_str* t3 = ov_str_concat(t2, p1);
  ov_str* t4 = ov_str_new("\"><input name=\"label\" placeholder=\"label\"> <button>", 51);
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_str* t6 = ov_str_concat(t5, p2);
  ov_str* t7 = ov_str_new("</button></form>", 16);
  ov_str* t8 = ov_str_concat(t6, t7);
  return t8;
}
