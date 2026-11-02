#include "ov_types.h"

int64_t ov_db_int_36007c3c_9a9bc110(int64_t, ov_str*, ov_list*);
int64_t ov_log_activity_e9ca7bb3_dab08e1e(int64_t, int64_t, ov_str*, ov_str*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
int64_t ov_add_comment_cccdec46_bf9a060b(int64_t p0, ovt_Issue_b825796a* p1, ovt_User_9c127ab2* p2, ov_str* p3);

int64_t ov_add_comment_cccdec46_bf9a060b(int64_t p0, ovt_Issue_b825796a* p1, ovt_User_9c127ab2* p2, ov_str* p3) {
  ov_str* t1 = ov_str_new("insert into comments (issue_id, author_id, body) values (\?, \?, \?) returning id", 78);
  int64_t t2 = p1->f_id;
  ov_str* t3 = ov_p_int_4640db5b_f21961d6(t2);
  int64_t t4 = p2->f_id;
  ov_str* t5 = ov_p_int_4640db5b_f21961d6(t4);
  ov_str* t6 = ov_p_str_73f85a82_17f92fe7(p3);
  ov_list* t7 = ov_list_new(3);
  t7->items[0] = (ov_value){.p = t3};
  t7->items[1] = (ov_value){.p = t5};
  t7->items[2] = (ov_value){.p = t6};
  int64_t t8 = ov_db_int_36007c3c_9a9bc110(p0, t1, t7);
  int64_t v2237 = t8;
  bool t9 = (v2237 > INT64_C(0));
  if (t9) {
    int64_t t10 = p1->f_id;
    ov_str* t11 = ov_str_new("commented", 9);
    ov_str* t12 = p2->f_name;
    int64_t t13 = ov_log_activity_e9ca7bb3_dab08e1e(p0, t10, t11, t12);
  }
  return v2237;
}
