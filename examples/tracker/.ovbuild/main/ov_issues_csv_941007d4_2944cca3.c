#include "ov_types.h"

ov_list* ov_map_9a617375_4c51e675(ov_list*, ov_closure*);
ov_list* ov_map_9a617375_6da7d9d8(ov_list*, ov_closure*);
ov_str* ov_csv_field_8bca961e_a63d9663(ov_str*);
ov_str* ov_issue_key_13b3fa8f_4e392611(ovt_Issue_b825796a*);
ov_str* ov_issues_csv_941007d4_2944cca3(ov_list* p0);

static ov_str* ovl_ov_issues_csv_941007d4_2944cca3_4482(void* ovenv, ov_str* l4482_0) {
  ov_str* t13 = ov_csv_field_8bca961e_a63d9663(l4482_0);
  return t13;
}

static ov_str* ovl_ov_issues_csv_941007d4_2944cca3_4488(void* ovenv, ovt_Issue_b825796a* l4488_0) {
  ov_str* t2 = ov_issue_key_13b3fa8f_4e392611(l4488_0);
  ov_str* t3 = l4488_0->f_title;
  ov_str* t4 = l4488_0->f_status;
  int64_t t5 = l4488_0->f_priority;
  ov_str* t6 = ov_show_int(t5);
  ov_str* t7 = l4488_0->f_assignee;
  ov_list* t8 = l4488_0->f_labels;
  ov_str* t9 = ov_str_new(";", 1);
  ov_str* t10 = ov_str_join(t8, t9);
  ov_str* t11 = l4488_0->f_body;
  ov_list* t12 = ov_list_new(7);
  t12->items[0] = (ov_value){.p = t2};
  t12->items[1] = (ov_value){.p = t3};
  t12->items[2] = (ov_value){.p = t4};
  t12->items[3] = (ov_value){.p = t6};
  t12->items[4] = (ov_value){.p = t7};
  t12->items[5] = (ov_value){.p = t10};
  t12->items[6] = (ov_value){.p = t11};
  ov_closure* t14 = ov_closure_mk((void*)ovl_ov_issues_csv_941007d4_2944cca3_4482, NULL);
  ov_list* t15 = ov_map_9a617375_6da7d9d8(t12, t14);
  ov_str* t16 = ov_str_new(",", 1);
  ov_str* t17 = ov_str_join(t15, t16);
  ov_str* t18 = ov_str_new("\n", 1);
  ov_str* t19 = ov_str_concat(t17, t18);
  return t19;
}

ov_str* ov_issues_csv_941007d4_2944cca3(ov_list* p0) {
  ov_str* t1 = ov_str_new("key,title,status,priority,assignee,labels,body\n", 47);
  ov_closure* t20 = ov_closure_mk((void*)ovl_ov_issues_csv_941007d4_2944cca3_4488, NULL);
  ov_list* t21 = ov_map_9a617375_4c51e675(p0, t20);
  ov_str* t22 = ov_str_new("", 0);
  ov_str* t23 = ov_str_join(t21, t22);
  ov_str* t24 = ov_str_concat(t1, t23);
  return t24;
}
