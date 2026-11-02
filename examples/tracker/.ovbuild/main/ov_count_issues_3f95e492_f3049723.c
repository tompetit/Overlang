#include "ov_types.h"

int64_t ov_db_int_36007c3c_9a9bc110(int64_t, ov_str*, ov_list*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ovt_tup2_24c1b644* ov_issue_where_8bfaaf6f_1a38c29a(ovt_Filter_50e09095*);
int64_t ov_count_issues_3f95e492_f3049723(int64_t p0, int64_t p1, ovt_Filter_50e09095* p2);

int64_t ov_count_issues_3f95e492_f3049723(int64_t p0, int64_t p1, ovt_Filter_50e09095* p2) {
  ovt_tup2_24c1b644* t1 = ov_issue_where_8bfaaf6f_1a38c29a(p2);
  ovt_tup2_24c1b644* v2003 = t1;
  ov_str* t2 = ov_str_new("select count(*) from issues i left join users u on u.id = i.assignee_id where i.project_id = \?", 94);
  ov_str* t3 = v2003->f0;
  ov_str* t4 = ov_str_concat(t2, t3);
  ov_str* v2007 = t4;
  ov_str* t5 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t6 = ov_list_new(1);
  t6->items[0] = (ov_value){.p = t5};
  ov_list* t7 = v2003->f1;
  ov_list* t8 = ov_list_concat(t6, t7);
  int64_t t9 = ov_db_int_36007c3c_9a9bc110(p0, v2007, t8);
  return t9;
}
