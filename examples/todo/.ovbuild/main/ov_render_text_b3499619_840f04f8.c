#include "ov_types.h"

ov_list* ov_map_080f5919_1b7fb3ea(ov_list*, ov_closure*);
ov_str* ov_render_text_b3499619_840f04f8(ov_list* p0);

static ov_str* ovl_ov_render_text_b3499619_840f04f8_540(void* ovenv, ovt_Todo_6462003c* l540_0) {
  int64_t t1 = l540_0->f_id;
  ov_str* t2 = ov_show_int(t1);
  ov_str* t3 = ov_str_new(" ", 1);
  ov_str* t4 = ov_str_concat(t2, t3);
  bool t5 = l540_0->f_done;
  ov_str* t6 = 0; (void)t6;
  if (t5) {
    ov_str* t7 = ov_str_new("[x] ", 4);
    t6 = t7;
  } else {
    ov_str* t8 = ov_str_new("[ ] ", 4);
    t6 = t8;
  }
  ov_str* t9 = ov_str_concat(t4, t6);
  ov_str* t10 = l540_0->f_title;
  ov_str* t11 = ov_str_concat(t9, t10);
  ov_str* t12 = ov_str_new("\n", 1);
  ov_str* t13 = ov_str_concat(t11, t12);
  return t13;
}

ov_str* ov_render_text_b3499619_840f04f8(ov_list* p0) {
  ov_closure* t14 = ov_closure_mk((void*)ovl_ov_render_text_b3499619_840f04f8_540, NULL);
  ov_list* t15 = ov_map_080f5919_1b7fb3ea(p0, t14);
  ov_str* t16 = ov_str_new("", 0);
  ov_str* t17 = ov_str_join(t15, t16);
  return t17;
}
