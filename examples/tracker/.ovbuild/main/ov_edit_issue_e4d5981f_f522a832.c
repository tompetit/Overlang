#include "ov_types.h"

int64_t ov_db_exec_3d590b5f_20515640(int64_t, ov_str*, ov_list*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
int64_t ov_edit_issue_e4d5981f_f522a832(int64_t p0, ovt_Issue_b825796a* p1, ov_str* p2, ov_str* p3, int64_t p4);

int64_t ov_edit_issue_e4d5981f_f522a832(int64_t p0, ovt_Issue_b825796a* p1, ov_str* p2, ov_str* p3, int64_t p4) {
  bool t1 = (p4 >= INT64_C(1));
  bool t2 = t1;
  if (t2) {
    bool t3 = (p4 <= INT64_C(4));
    t2 = t3;
  }
  if (!t2) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"i\": ");
    ov_sb_push(ovsb, ovsup_json_str(ovshow_Issue_b825796a(p1)));
    ov_sb_push_cstr(ovsb, ", \"title\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p2)));
    ov_sb_push_cstr(ovsb, ", \"body\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p3)));
    ov_sb_push_cstr(ovsb, ", \"priority\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p4)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "edit_issue", "e4d5981f38f9c9b680cd93f6052a3e7f8e84f5a97072dc017a8dbcad5ab0c732", "priority >= 1 && priority <= 4", ov_sb_finish(ovsb)->data);
  }
  ov_str* t4 = ov_str_new("update issues set title = \?, body = \?, priority = \? where id = \?", 64);
  ov_str* t5 = ov_p_str_73f85a82_17f92fe7(p2);
  ov_str* t6 = ov_p_str_73f85a82_17f92fe7(p3);
  ov_str* t7 = ov_p_int_4640db5b_f21961d6(p4);
  int64_t t8 = p1->f_id;
  ov_str* t9 = ov_p_int_4640db5b_f21961d6(t8);
  ov_list* t10 = ov_list_new(4);
  t10->items[0] = (ov_value){.p = t5};
  t10->items[1] = (ov_value){.p = t6};
  t10->items[2] = (ov_value){.p = t7};
  t10->items[3] = (ov_value){.p = t9};
  int64_t t11 = ov_db_exec_3d590b5f_20515640(p0, t4, t10);
  return t11;
}
