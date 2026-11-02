#include "ov_types.h"

ov_str* ov_dash_if_empty_4d9d1960_b97c4fb0(ov_str*);
ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str*);
ov_str* ov_issue_key_13b3fa8f_4e392611(ovt_Issue_b825796a*);
ov_str* ov_labels_display_122cd589_821b1221(ov_list*);
ov_str* ov_render_issue_row_2d3f3036_72855283(ovt_Issue_b825796a* p0);

ov_str* ov_render_issue_row_2d3f3036_72855283(ovt_Issue_b825796a* p0) {
  ov_str* t1 = ov_str_new("<tr><td><a href=\"/p/", 20);
  ov_str* t2 = p0->f_project_key;
  ov_str* t3 = ov_html_escape_114cb911_d5a1eb27(t2);
  ov_str* t4 = ov_str_concat(t1, t3);
  ov_str* t5 = ov_str_new("/", 1);
  ov_str* t6 = ov_str_concat(t4, t5);
  int64_t t7 = p0->f_number;
  ov_str* t8 = ov_show_int(t7);
  ov_str* t9 = ov_str_concat(t6, t8);
  ov_str* t10 = ov_str_new("\">", 2);
  ov_str* t11 = ov_str_concat(t9, t10);
  ov_str* t12 = ov_issue_key_13b3fa8f_4e392611(p0);
  ov_str* t13 = ov_html_escape_114cb911_d5a1eb27(t12);
  ov_str* t14 = ov_str_concat(t11, t13);
  ov_str* t15 = ov_str_new("</a></td><td>", 13);
  ov_str* t16 = ov_str_concat(t14, t15);
  ov_str* t17 = p0->f_title;
  ov_str* t18 = ov_html_escape_114cb911_d5a1eb27(t17);
  ov_str* t19 = ov_str_concat(t16, t18);
  ov_str* t20 = ov_str_new("</td><td>", 9);
  ov_str* t21 = ov_str_concat(t19, t20);
  ov_str* t22 = p0->f_status;
  ov_str* t23 = ov_html_escape_114cb911_d5a1eb27(t22);
  ov_str* t24 = ov_str_concat(t21, t23);
  ov_str* t25 = ov_str_new("</td><td>P", 10);
  ov_str* t26 = ov_str_concat(t24, t25);
  int64_t t27 = p0->f_priority;
  ov_str* t28 = ov_show_int(t27);
  ov_str* t29 = ov_str_concat(t26, t28);
  ov_str* t30 = ov_str_new("</td><td>", 9);
  ov_str* t31 = ov_str_concat(t29, t30);
  ov_str* t32 = p0->f_assignee;
  ov_str* t33 = ov_dash_if_empty_4d9d1960_b97c4fb0(t32);
  ov_str* t34 = ov_html_escape_114cb911_d5a1eb27(t33);
  ov_str* t35 = ov_str_concat(t31, t34);
  ov_str* t36 = ov_str_new("</td><td>", 9);
  ov_str* t37 = ov_str_concat(t35, t36);
  ov_list* t38 = p0->f_labels;
  ov_str* t39 = ov_labels_display_122cd589_821b1221(t38);
  ov_str* t40 = ov_html_escape_114cb911_d5a1eb27(t39);
  ov_str* t41 = ov_str_concat(t37, t40);
  ov_str* t42 = ov_str_new("</td></tr>", 10);
  ov_str* t43 = ov_str_concat(t41, t42);
  return t43;
}
