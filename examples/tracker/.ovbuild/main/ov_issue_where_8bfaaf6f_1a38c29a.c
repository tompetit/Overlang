#include "ov_types.h"

ov_str* ov_like_pattern_ec16774e_b5007031(ov_str*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
ovt_tup2_24c1b644* ov_issue_where_8bfaaf6f_1a38c29a(ovt_Filter_50e09095* p0);

ovt_tup2_24c1b644* ov_issue_where_8bfaaf6f_1a38c29a(ovt_Filter_50e09095* p0) {
  ov_str* t1 = ov_str_new("", 0);
  ov_str* v1676 = t1;
  (void)v1676;
  ov_list* t2 = ov_list_empty();
  ov_list* v1678 = t2;
  (void)v1678;
  ov_str* t3 = p0->f_status;
  ov_str* t4 = ov_str_new("", 0);
  bool t5 = (!ov_str_eq(t3, t4));
  if (t5) {
    ov_str* t6 = v1676;
    ov_str* t7 = ov_str_new(" and i.status = \?", 17);
    ov_str* t8 = ov_str_concat(t6, t7);
    v1676 = t8;
    ov_list* t9 = v1678;
    ov_str* t10 = p0->f_status;
    ov_str* t11 = ov_p_str_73f85a82_17f92fe7(t10);
    ov_list* t12 = ov_list_push(t9, (ov_value){.p = t11});
    v1678 = t12;
  }
  ov_str* t13 = p0->f_assignee;
  ov_str* t14 = ov_str_new("", 0);
  bool t15 = (!ov_str_eq(t13, t14));
  if (t15) {
    ov_str* t16 = v1676;
    ov_str* t17 = ov_str_new(" and u.name = \?", 15);
    ov_str* t18 = ov_str_concat(t16, t17);
    v1676 = t18;
    ov_list* t19 = v1678;
    ov_str* t20 = p0->f_assignee;
    ov_str* t21 = ov_p_str_73f85a82_17f92fe7(t20);
    ov_list* t22 = ov_list_push(t19, (ov_value){.p = t21});
    v1678 = t22;
  }
  ov_str* t23 = p0->f_label;
  ov_str* t24 = ov_str_new("", 0);
  bool t25 = (!ov_str_eq(t23, t24));
  if (t25) {
    ov_str* t26 = v1676;
    ov_str* t27 = ov_str_new(" and exists (select 1 from issue_labels il2 join labels l2 on l2.id = il2.label_id where il2.issue_id = i.id and l2.name = \?)", 125);
    ov_str* t28 = ov_str_concat(t26, t27);
    v1676 = t28;
    ov_list* t29 = v1678;
    ov_str* t30 = p0->f_label;
    ov_str* t31 = ov_p_str_73f85a82_17f92fe7(t30);
    ov_list* t32 = ov_list_push(t29, (ov_value){.p = t31});
    v1678 = t32;
  }
  int64_t t33 = p0->f_priority;
  bool t34 = (t33 > INT64_C(0));
  if (t34) {
    ov_str* t35 = v1676;
    ov_str* t36 = ov_str_new(" and i.priority = \?", 19);
    ov_str* t37 = ov_str_concat(t35, t36);
    v1676 = t37;
    ov_list* t38 = v1678;
    int64_t t39 = p0->f_priority;
    ov_str* t40 = ov_p_int_4640db5b_f21961d6(t39);
    ov_list* t41 = ov_list_push(t38, (ov_value){.p = t40});
    v1678 = t41;
  }
  ov_str* t42 = p0->f_q;
  ov_str* t43 = ov_str_new("", 0);
  bool t44 = (!ov_str_eq(t42, t43));
  if (t44) {
    ov_str* t45 = v1676;
    ov_str* t46 = ov_str_new(" and (i.title like \? escape '\\' or i.body like \? escape '\\')", 60);
    ov_str* t47 = ov_str_concat(t45, t46);
    v1676 = t47;
    ov_list* t48 = v1678;
    ov_str* t49 = p0->f_q;
    ov_str* t50 = ov_like_pattern_ec16774e_b5007031(t49);
    ov_str* t51 = ov_p_str_73f85a82_17f92fe7(t50);
    ov_str* t52 = p0->f_q;
    ov_str* t53 = ov_like_pattern_ec16774e_b5007031(t52);
    ov_str* t54 = ov_p_str_73f85a82_17f92fe7(t53);
    ov_list* t55 = ov_list_new(2);
    t55->items[0] = (ov_value){.p = t51};
    t55->items[1] = (ov_value){.p = t54};
    ov_list* t56 = ov_list_concat(t48, t55);
    v1678 = t56;
  }
  ov_str* t57 = v1676;
  ov_list* t58 = v1678;
  ovt_tup2_24c1b644* t59 = ovt_tup2_24c1b644_mk(t57, t58);
  return t59;
}
