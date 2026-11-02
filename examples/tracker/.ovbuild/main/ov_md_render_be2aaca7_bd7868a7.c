#include "ov_types.h"

ov_list* ov_lines_70f746c7_d92a2e51(ov_str*);
ov_list* ov_map_9a617375_6da7d9d8(ov_list*, ov_closure*);
ov_str* ov_md_inline_c5ca95ff_9e3bf05f(ov_str*);
ov_str* ov_md_render_be2aaca7_bd7868a7(ov_str* p0);

static ov_str* ovl_ov_md_render_be2aaca7_bd7868a7_3598(void* ovenv, ov_str* l3598_0) {
  ov_str* t31 = ov_str_new("<p>", 3);
  ov_str* t32 = ov_md_inline_c5ca95ff_9e3bf05f(l3598_0);
  ov_str* t33 = ov_str_concat(t31, t32);
  ov_str* t34 = ov_str_new("</p>", 4);
  ov_str* t35 = ov_str_concat(t33, t34);
  return t35;
}

ov_str* ov_md_render_be2aaca7_bd7868a7(ov_str* p0) {
  ov_list* t1 = ov_list_empty();
  ov_list* v3542 = t1;
  (void)v3542;
  ov_list* t2 = ov_list_empty();
  ov_list* v3544 = t2;
  (void)v3544;
  ov_list* t3 = ov_lines_70f746c7_d92a2e51(p0);
  ov_list* t4 = t3;
  for (int64_t t5 = 0; t5 < t4->len; t5++) {
    ov_str* t6 = ((ov_str*)(t4->items[t5]).p);
    ov_str* v3546 = t6;
    ov_str* t8 = ov_str_trim(v3546);
    ov_str* t9 = ov_str_new("", 0);
    bool t10 = ov_str_eq(t8, t9);
    if (t10) {
      ov_list* t11 = v3544;
      int64_t t12 = t11->len;
      bool t13 = (t12 > INT64_C(0));
      if (t13) {
        ov_list* t14 = v3542;
        ov_list* t15 = v3544;
        ov_str* t16 = ov_str_new("\n", 1);
        ov_str* t17 = ov_str_join(t15, t16);
        ov_list* t18 = ov_list_push(t14, (ov_value){.p = t17});
        v3542 = t18;
        ov_list* t19 = ov_list_empty();
        v3544 = t19;
      }
    } else {
      ov_list* t20 = v3544;
      ov_list* t21 = ov_list_push(t20, (ov_value){.p = v3546});
      v3544 = t21;
    }
  }
  ov_list* t22 = v3544;
  int64_t t23 = t22->len;
  bool t24 = (t23 > INT64_C(0));
  if (t24) {
    ov_list* t25 = v3542;
    ov_list* t26 = v3544;
    ov_str* t27 = ov_str_new("\n", 1);
    ov_str* t28 = ov_str_join(t26, t27);
    ov_list* t29 = ov_list_push(t25, (ov_value){.p = t28});
    v3542 = t29;
  }
  ov_list* t30 = v3542;
  ov_closure* t36 = ov_closure_mk((void*)ovl_ov_md_render_be2aaca7_bd7868a7_3598, NULL);
  ov_list* t37 = ov_map_9a617375_6da7d9d8(t30, t36);
  ov_str* t38 = ov_str_new("\n", 1);
  ov_str* t39 = ov_str_join(t37, t38);
  return t39;
}
