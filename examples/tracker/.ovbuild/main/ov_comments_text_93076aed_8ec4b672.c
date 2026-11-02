#include "ov_types.h"

ov_list* ov_map_9a617375_9a84effc(ov_list*, ov_closure*);
ov_str* ov_comments_text_93076aed_8ec4b672(ov_list* p0);

static ov_str* ovl_ov_comments_text_93076aed_8ec4b672_4308(void* ovenv, ovt_Comment_21df0379* l4308_0) {
  ov_str* t1 = l4308_0->f_author;
  ov_str* t2 = ov_str_new(": ", 2);
  ov_str* t3 = ov_str_concat(t1, t2);
  ov_str* t4 = l4308_0->f_body;
  ov_str* t5 = ov_str_new("\n", 1);
  ov_str* t6 = ov_str_new(" ", 1);
  ov_str* t7 = ov_str_replace(t4, t5, t6);
  ov_str* t8 = ov_str_concat(t3, t7);
  ov_str* t9 = ov_str_new("\n", 1);
  ov_str* t10 = ov_str_concat(t8, t9);
  return t10;
}

ov_str* ov_comments_text_93076aed_8ec4b672(ov_list* p0) {
  ov_closure* t11 = ov_closure_mk((void*)ovl_ov_comments_text_93076aed_8ec4b672_4308, NULL);
  ov_list* t12 = ov_map_9a617375_9a84effc(p0, t11);
  ov_str* t13 = ov_str_new("", 0);
  ov_str* t14 = ov_str_join(t12, t13);
  return t14;
}
