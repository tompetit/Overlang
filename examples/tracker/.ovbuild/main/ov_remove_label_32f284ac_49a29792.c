#include "ov_types.h"

int64_t ov_db_exec_3d590b5f_20515640(int64_t, ov_str*, ov_list*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
int64_t ov_remove_label_32f284ac_49a29792(int64_t p0, ovt_Issue_b825796a* p1, ovt_Label_da07b2c4* p2);

int64_t ov_remove_label_32f284ac_49a29792(int64_t p0, ovt_Issue_b825796a* p1, ovt_Label_da07b2c4* p2) {
  ov_str* t1 = ov_str_new("delete from issue_labels where issue_id = \? and label_id = \?", 60);
  int64_t t2 = p1->f_id;
  ov_str* t3 = ov_p_int_4640db5b_f21961d6(t2);
  int64_t t4 = p2->f_id;
  ov_str* t5 = ov_p_int_4640db5b_f21961d6(t4);
  ov_list* t6 = ov_list_new(2);
  t6->items[0] = (ov_value){.p = t3};
  t6->items[1] = (ov_value){.p = t5};
  int64_t t7 = ov_db_exec_3d590b5f_20515640(p0, t1, t6);
  return t7;
}
