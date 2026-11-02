#include "ov_types.h"

int64_t ov_db_exec_3d590b5f_20515640(int64_t, ov_str*, ov_list*);
int64_t ov_log_activity_e9ca7bb3_dab08e1e(int64_t, int64_t, ov_str*, ov_str*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_null_a6460182_0d7cc796(void);
bool ov_set_assignee_df36439c_2f12287e(int64_t p0, ovt_Issue_b825796a* p1, ovt_Option_7fe81862* p2);

bool ov_set_assignee_df36439c_2f12287e(int64_t p0, ovt_Issue_b825796a* p1, ovt_Option_7fe81862* p2) {
  ovt_tup2_b1cce6e4* t1 = 0; (void)t1;
  bool t2 = false; (void)t2;
  if (!t2) {
    if ((p2->tag == 0)) {
      ovt_User_9c127ab2* v2165 = p2->u.c0.f0;
      t2 = true;
      ov_str* t3 = v2165->f_name;
      int64_t t4 = v2165->f_id;
      ov_str* t5 = ov_p_int_4640db5b_f21961d6(t4);
      ovt_tup2_b1cce6e4* t6 = ovt_tup2_b1cce6e4_mk(t3, t5);
      t1 = t6;
    }
  }
  if (!t2) {
    if ((p2->tag == 1)) {
      t2 = true;
      ov_str* t7 = ov_str_new("", 0);
      ov_str* t8 = ov_p_null_a6460182_0d7cc796();
      ovt_tup2_b1cce6e4* t9 = ovt_tup2_b1cce6e4_mk(t7, t8);
      t1 = t9;
    }
  }
  ov_str* v2161 = t1->f0;
  ov_str* v2162 = t1->f1;
  ov_str* t10 = p1->f_assignee;
  bool t11 = ov_str_eq(v2161, t10);
  bool t12 = 0; (void)t12;
  if (t11) {
    t12 = false;
  } else {
    ov_str* t13 = ov_str_new("update issues set assignee_id = \? where id = \?", 46);
    int64_t t14 = p1->f_id;
    ov_str* t15 = ov_p_int_4640db5b_f21961d6(t14);
    ov_list* t16 = ov_list_new(2);
    t16->items[0] = (ov_value){.p = v2162};
    t16->items[1] = (ov_value){.p = t15};
    int64_t t17 = ov_db_exec_3d590b5f_20515640(p0, t13, t16);
    int64_t t18 = p1->f_id;
    ov_str* t19 = ov_str_new("assigned", 8);
    ov_str* t20 = ov_str_new("", 0);
    bool t21 = ov_str_eq(v2161, t20);
    ov_str* t22 = 0; (void)t22;
    if (t21) {
      ov_str* t23 = ov_str_new("-", 1);
      t22 = t23;
    } else {
      t22 = v2161;
    }
    int64_t t24 = ov_log_activity_e9ca7bb3_dab08e1e(p0, t18, t19, t22);
    t12 = true;
  }
  return t12;
}
