#include "ov_types.h"

ov_str* ov_issue_key_13b3fa8f_4e392611(ovt_Issue_b825796a* p0);

ov_str* ov_issue_key_13b3fa8f_4e392611(ovt_Issue_b825796a* p0) {
  ov_str* t1 = p0->f_project_key;
  ov_str* t2 = ov_str_new("-", 1);
  ov_str* t3 = ov_str_concat(t1, t2);
  int64_t t4 = p0->f_number;
  ov_str* t5 = ov_show_int(t4);
  ov_str* t6 = ov_str_concat(t3, t5);
  return t6;
}
