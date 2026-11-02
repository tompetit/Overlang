#include "ov_types.h"

ov_list* ov_map_9a617375_6da7d9d8(ov_list*, ov_closure*);
ov_list* ov_map_9a617375_7a67dbbf(ov_list*, ov_closure*);
ov_list* ov_map_9a617375_9a84effc(ov_list*, ov_closure*);
ov_str* ov_issue_key_13b3fa8f_4e392611(ovt_Issue_b825796a*);
ov_str* ov_json_str_a498952b_d002ebb1(ov_str*);
ov_str* ov_issue_json_b69e0135_43ca48f1(ovt_Issue_b825796a* p0, ov_list* p1, ov_list* p2);

static ov_str* ovl_ov_issue_json_b69e0135_43ca48f1_4396(void* ovenv, ov_str* l4396_0) {
  ov_str* t32 = ov_json_str_a498952b_d002ebb1(l4396_0);
  return t32;
}

static ov_str* ovl_ov_issue_json_b69e0135_43ca48f1_4420(void* ovenv, ovt_Comment_21df0379* l4420_0) {
  ov_str* t39 = ov_str_new("{\"author\":", 10);
  ov_str* t40 = l4420_0->f_author;
  ov_str* t41 = ov_json_str_a498952b_d002ebb1(t40);
  ov_str* t42 = ov_str_concat(t39, t41);
  ov_str* t43 = ov_str_new(",\"body\":", 8);
  ov_str* t44 = ov_str_concat(t42, t43);
  ov_str* t45 = l4420_0->f_body;
  ov_str* t46 = ov_json_str_a498952b_d002ebb1(t45);
  ov_str* t47 = ov_str_concat(t44, t46);
  ov_str* t48 = ov_str_new("}", 1);
  ov_str* t49 = ov_str_concat(t47, t48);
  return t49;
}

static ov_str* ovl_ov_issue_json_b69e0135_43ca48f1_4444(void* ovenv, ovt_Activity_dd4dbed0* l4444_0) {
  ov_str* t56 = ov_str_new("{\"kind\":", 8);
  ov_str* t57 = l4444_0->f_kind;
  ov_str* t58 = ov_json_str_a498952b_d002ebb1(t57);
  ov_str* t59 = ov_str_concat(t56, t58);
  ov_str* t60 = ov_str_new(",\"detail\":", 10);
  ov_str* t61 = ov_str_concat(t59, t60);
  ov_str* t62 = l4444_0->f_detail;
  ov_str* t63 = ov_json_str_a498952b_d002ebb1(t62);
  ov_str* t64 = ov_str_concat(t61, t63);
  ov_str* t65 = ov_str_new("}", 1);
  ov_str* t66 = ov_str_concat(t64, t65);
  return t66;
}

ov_str* ov_issue_json_b69e0135_43ca48f1(ovt_Issue_b825796a* p0, ov_list* p1, ov_list* p2) {
  ov_str* t1 = ov_str_new("{\"key\":", 7);
  ov_str* t2 = ov_issue_key_13b3fa8f_4e392611(p0);
  ov_str* t3 = ov_json_str_a498952b_d002ebb1(t2);
  ov_str* t4 = ov_str_new(",\"project\":", 11);
  ov_str* t5 = p0->f_project_key;
  ov_str* t6 = ov_json_str_a498952b_d002ebb1(t5);
  ov_str* t7 = ov_str_new(",\"number\":", 10);
  int64_t t8 = p0->f_number;
  ov_str* t9 = ov_show_int(t8);
  ov_str* t10 = ov_str_new(",\"title\":", 9);
  ov_str* t11 = p0->f_title;
  ov_str* t12 = ov_json_str_a498952b_d002ebb1(t11);
  ov_str* t13 = ov_str_new(",\"body\":", 8);
  ov_str* t14 = p0->f_body;
  ov_str* t15 = ov_json_str_a498952b_d002ebb1(t14);
  ov_str* t16 = ov_str_new(",\"status\":", 10);
  ov_str* t17 = p0->f_status;
  ov_str* t18 = ov_json_str_a498952b_d002ebb1(t17);
  ov_str* t19 = ov_str_new(",\"priority\":", 12);
  int64_t t20 = p0->f_priority;
  ov_str* t21 = ov_show_int(t20);
  ov_str* t22 = ov_str_new(",\"assignee\":", 12);
  ov_str* t23 = p0->f_assignee;
  ov_str* t24 = ov_str_new("", 0);
  bool t25 = ov_str_eq(t23, t24);
  ov_str* t26 = 0; (void)t26;
  if (t25) {
    ov_str* t27 = ov_str_new("null", 4);
    t26 = t27;
  } else {
    ov_str* t28 = p0->f_assignee;
    ov_str* t29 = ov_json_str_a498952b_d002ebb1(t28);
    t26 = t29;
  }
  ov_str* t30 = ov_str_new(",\"labels\":[", 11);
  ov_list* t31 = p0->f_labels;
  ov_closure* t33 = ov_closure_mk((void*)ovl_ov_issue_json_b69e0135_43ca48f1_4396, NULL);
  ov_list* t34 = ov_map_9a617375_6da7d9d8(t31, t33);
  ov_str* t35 = ov_str_new(",", 1);
  ov_str* t36 = ov_str_join(t34, t35);
  ov_str* t37 = ov_str_new("]", 1);
  ov_str* t38 = ov_str_new(",\"comments\":[", 13);
  ov_closure* t50 = ov_closure_mk((void*)ovl_ov_issue_json_b69e0135_43ca48f1_4420, NULL);
  ov_list* t51 = ov_map_9a617375_9a84effc(p1, t50);
  ov_str* t52 = ov_str_new(",", 1);
  ov_str* t53 = ov_str_join(t51, t52);
  ov_str* t54 = ov_str_new("]", 1);
  ov_str* t55 = ov_str_new(",\"activity\":[", 13);
  ov_closure* t67 = ov_closure_mk((void*)ovl_ov_issue_json_b69e0135_43ca48f1_4444, NULL);
  ov_list* t68 = ov_map_9a617375_7a67dbbf(p2, t67);
  ov_str* t69 = ov_str_new(",", 1);
  ov_str* t70 = ov_str_join(t68, t69);
  ov_str* t71 = ov_str_new("]}\n", 3);
  ov_list* t72 = ov_list_new(25);
  t72->items[0] = (ov_value){.p = t1};
  t72->items[1] = (ov_value){.p = t3};
  t72->items[2] = (ov_value){.p = t4};
  t72->items[3] = (ov_value){.p = t6};
  t72->items[4] = (ov_value){.p = t7};
  t72->items[5] = (ov_value){.p = t9};
  t72->items[6] = (ov_value){.p = t10};
  t72->items[7] = (ov_value){.p = t12};
  t72->items[8] = (ov_value){.p = t13};
  t72->items[9] = (ov_value){.p = t15};
  t72->items[10] = (ov_value){.p = t16};
  t72->items[11] = (ov_value){.p = t18};
  t72->items[12] = (ov_value){.p = t19};
  t72->items[13] = (ov_value){.p = t21};
  t72->items[14] = (ov_value){.p = t22};
  t72->items[15] = (ov_value){.p = t26};
  t72->items[16] = (ov_value){.p = t30};
  t72->items[17] = (ov_value){.p = t36};
  t72->items[18] = (ov_value){.p = t37};
  t72->items[19] = (ov_value){.p = t38};
  t72->items[20] = (ov_value){.p = t53};
  t72->items[21] = (ov_value){.p = t54};
  t72->items[22] = (ov_value){.p = t55};
  t72->items[23] = (ov_value){.p = t70};
  t72->items[24] = (ov_value){.p = t71};
  ov_str* t73 = ov_str_new("", 0);
  ov_str* t74 = ov_str_join(t72, t73);
  return t74;
}
