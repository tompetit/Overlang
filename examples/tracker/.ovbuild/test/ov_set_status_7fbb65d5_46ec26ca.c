#include "ov_types.h"

bool ov_valid_status_8b9994c4_c3c13714(ov_str*);
int64_t ov_db_exec_3d590b5f_20515640(int64_t, ov_str*, ov_list*);
int64_t ov_log_activity_e9ca7bb3_dab08e1e(int64_t, int64_t, ov_str*, ov_str*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
bool ov_set_status_7fbb65d5_46ec26ca(int64_t p0, ovt_Issue_b825796a* p1, ov_str* p2);

bool ov_set_status_7fbb65d5_46ec26ca(int64_t p0, ovt_Issue_b825796a* p1, ov_str* p2) {
  bool t1 = ov_valid_status_8b9994c4_c3c13714(p2);
  if (!t1) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"i\": ");
    ov_sb_push(ovsb, ovsup_json_str(ovshow_Issue_b825796a(p1)));
    ov_sb_push_cstr(ovsb, ", \"status\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p2)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "set_status", "7fbb65d5380a13a8481cbc21945bac036e1026f46196a69043b76367edb925b8", "valid_status(status)", ov_sb_finish(ovsb)->data);
  }
  ov_str* t2 = p1->f_status;
  bool t3 = ov_str_eq(p2, t2);
  bool t4 = 0; (void)t4;
  if (t3) {
    t4 = false;
  } else {
    ov_str* t5 = ov_str_new("update issues set status = \? where id = \?", 41);
    ov_str* t6 = ov_p_str_73f85a82_17f92fe7(p2);
    int64_t t7 = p1->f_id;
    ov_str* t8 = ov_p_int_4640db5b_f21961d6(t7);
    ov_list* t9 = ov_list_new(2);
    t9->items[0] = (ov_value){.p = t6};
    t9->items[1] = (ov_value){.p = t8};
    int64_t t10 = ov_db_exec_3d590b5f_20515640(p0, t5, t9);
    int64_t t11 = p1->f_id;
    ov_str* t12 = ov_str_new("status", 6);
    int64_t t13 = ov_log_activity_e9ca7bb3_dab08e1e(p0, t11, t12, p2);
    t4 = true;
  }
  return t4;
}
