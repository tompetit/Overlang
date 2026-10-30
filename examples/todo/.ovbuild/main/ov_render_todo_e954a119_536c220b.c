#include "ov_types.h"

ov_str* ov_html_escape_bb603171_d5a1eb27(ov_str*);
ov_str* ov_id_form_f3edd237_f317a921(ov_str*, int64_t, ov_str*);
ov_str* ov_render_todo_e954a119_536c220b(ovt_Todo_6462003c* p0);

ov_str* ov_render_todo_e954a119_536c220b(ovt_Todo_6462003c* p0) {
  bool t1 = p0->f_done;
  ov_str* t2 = 0; (void)t2;
  if (t1) {
    ov_str* t3 = ov_str_new("[x] ", 4);
    t2 = t3;
  } else {
    ov_str* t4 = ov_str_new("[ ] ", 4);
    t2 = t4;
  }
  ov_str* v466 = t2;
  ov_str* t5 = ov_str_new("<li>", 4);
  ov_str* t6 = ov_str_concat(t5, v466);
  ov_str* t7 = p0->f_title;
  ov_str* t8 = ov_html_escape_bb603171_d5a1eb27(t7);
  ov_str* t9 = ov_str_concat(t6, t8);
  ov_str* t10 = ov_str_new(" ", 1);
  ov_str* t11 = ov_str_concat(t9, t10);
  ov_str* t12 = ov_str_new("/toggle", 7);
  int64_t t13 = p0->f_id;
  ov_str* t14 = ov_str_new("toggle", 6);
  ov_str* t15 = ov_id_form_f3edd237_f317a921(t12, t13, t14);
  ov_str* t16 = ov_str_concat(t11, t15);
  ov_str* t17 = ov_str_new(" ", 1);
  ov_str* t18 = ov_str_concat(t16, t17);
  ov_str* t19 = ov_str_new("/delete", 7);
  int64_t t20 = p0->f_id;
  ov_str* t21 = ov_str_new("delete", 6);
  ov_str* t22 = ov_id_form_f3edd237_f317a921(t19, t20, t21);
  ov_str* t23 = ov_str_concat(t18, t22);
  ov_str* t24 = ov_str_new("</li>", 5);
  ov_str* t25 = ov_str_concat(t23, t24);
  return t25;
}
