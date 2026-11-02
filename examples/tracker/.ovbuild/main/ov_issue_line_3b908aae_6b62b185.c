#include "ov_types.h"

ov_str* ov_dash_if_empty_4d9d1960_b97c4fb0(ov_str*);
ov_str* ov_issue_key_13b3fa8f_4e392611(ovt_Issue_b825796a*);
ov_str* ov_issue_line_3b908aae_6b62b185(ovt_Issue_b825796a* p0);

ov_str* ov_issue_line_3b908aae_6b62b185(ovt_Issue_b825796a* p0) {
  ov_str* t1 = ov_issue_key_13b3fa8f_4e392611(p0);
  ov_str* t2 = ov_str_new(" ", 1);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = p0->f_status;
  ov_str* t5 = ov_str_concat(t3, t4);
  ov_str* t6 = ov_str_new(" P", 2);
  ov_str* t7 = ov_str_concat(t5, t6);
  int64_t t8 = p0->f_priority;
  ov_str* t9 = ov_show_int(t8);
  ov_str* t10 = ov_str_concat(t7, t9);
  ov_str* t11 = ov_str_new(" ", 1);
  ov_str* t12 = ov_str_concat(t10, t11);
  ov_str* t13 = p0->f_assignee;
  ov_str* t14 = ov_dash_if_empty_4d9d1960_b97c4fb0(t13);
  ov_str* t15 = ov_str_concat(t12, t14);
  ov_str* t16 = ov_str_new(" [", 2);
  ov_str* t17 = ov_str_concat(t15, t16);
  ov_list* t18 = p0->f_labels;
  ov_str* t19 = ov_str_new(",", 1);
  ov_str* t20 = ov_str_join(t18, t19);
  ov_str* t21 = ov_str_concat(t17, t20);
  ov_str* t22 = ov_str_new("] ", 2);
  ov_str* t23 = ov_str_concat(t21, t22);
  ov_str* t24 = p0->f_title;
  ov_str* t25 = ov_str_concat(t23, t24);
  ov_str* t26 = ov_str_new("\n", 1);
  ov_str* t27 = ov_str_concat(t25, t26);
  return t27;
}
