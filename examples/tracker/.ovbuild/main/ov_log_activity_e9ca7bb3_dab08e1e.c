#include "ov_types.h"

int64_t ov_db_int_36007c3c_9a9bc110(int64_t, ov_str*, ov_list*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
int64_t ov_log_activity_e9ca7bb3_dab08e1e(int64_t p0, int64_t p1, ov_str* p2, ov_str* p3);

int64_t ov_log_activity_e9ca7bb3_dab08e1e(int64_t p0, int64_t p1, ov_str* p2, ov_str* p3) {
  ov_str* t1 = ov_str_new("insert into activity (issue_id, kind, detail) values (\?, \?, \?) returning id", 75);
  ov_str* t2 = ov_p_int_4640db5b_f21961d6(p1);
  ov_str* t3 = ov_p_str_73f85a82_17f92fe7(p2);
  ov_str* t4 = ov_p_str_73f85a82_17f92fe7(p3);
  ov_list* t5 = ov_list_new(3);
  t5->items[0] = (ov_value){.p = t2};
  t5->items[1] = (ov_value){.p = t3};
  t5->items[2] = (ov_value){.p = t4};
  int64_t t6 = ov_db_int_36007c3c_9a9bc110(p0, t1, t5);
  return t6;
}
