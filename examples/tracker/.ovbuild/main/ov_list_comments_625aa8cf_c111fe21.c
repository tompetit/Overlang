#include "ov_types.h"

ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_list* ov_map_9a617375_20d8fca3(ov_list*, ov_closure*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_Comment_21df0379* ov_row_comment_7c0ac623_ee6f869e(ov_list*);
ov_list* ov_list_comments_625aa8cf_c111fe21(int64_t p0, int64_t p1);

static ovt_Comment_21df0379* ovl_ov_list_comments_625aa8cf_c111fe21_2281(void* ovenv, ov_list* l2281_0) {
  ovt_Comment_21df0379* t5 = ov_row_comment_7c0ac623_ee6f869e(l2281_0);
  return t5;
}

ov_list* ov_list_comments_625aa8cf_c111fe21(int64_t p0, int64_t p1) {
  ov_str* t1 = ov_str_new("select c.id, c.issue_id, u.name, c.body from comments c join users u on u.id = c.author_id where c.issue_id = \? order by c.id", 125);
  ov_str* v2267 = t1;
  ov_str* t2 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t3 = ov_list_new(1);
  t3->items[0] = (ov_value){.p = t2};
  ov_list* t4 = ov_db_rows_2c7fa5b5_89b40a99(p0, v2267, t3);
  ov_closure* t6 = ov_closure_mk((void*)ovl_ov_list_comments_625aa8cf_c111fe21_2281, NULL);
  ov_list* t7 = ov_map_9a617375_20d8fca3(t4, t6);
  return t7;
}
