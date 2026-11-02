#include "ov_types.h"

int64_t ov_db_exec_3d590b5f_20515640(int64_t, ov_str*, ov_list*);
int64_t ov_db_int_36007c3c_9a9bc110(int64_t, ov_str*, ov_list*);
int64_t ov_log_activity_e9ca7bb3_dab08e1e(int64_t, int64_t, ov_str*, ov_str*);
ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_null_a6460182_0d7cc796(void);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
int64_t ov_create_issue_c9275d15_8de03dd4(int64_t p0, int64_t p1, ov_str* p2, ov_str* p3, int64_t p4, int64_t p5, ov_list* p6);

int64_t ov_create_issue_c9275d15_8de03dd4(int64_t p0, int64_t p1, ov_str* p2, ov_str* p3, int64_t p4, int64_t p5, ov_list* p6) {
  ov_str* t1 = ov_str_trim(p2);
  int64_t t2 = ov_str_len_cps(t1);
  bool t3 = (t2 > INT64_C(0));
  bool t4 = t3;
  if (t4) {
    bool t5 = (p4 >= INT64_C(1));
    t4 = t5;
  }
  bool t6 = t4;
  if (t6) {
    bool t7 = (p4 <= INT64_C(4));
    t6 = t7;
  }
  if (!t6) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"db\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"project_id\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p1)));
    ov_sb_push_cstr(ovsb, ", \"title\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p2)));
    ov_sb_push_cstr(ovsb, ", \"body\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p3)));
    ov_sb_push_cstr(ovsb, ", \"priority\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p4)));
    ov_sb_push_cstr(ovsb, ", \"assignee_id\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p5)));
    ov_sb_push_cstr(ovsb, ", \"label_ids\": ");
    ov_sb_push(ovsb, ovsup_json_str(ovshow_List_d31dced1(p6)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "create_issue", "c9275d15243ee94ce20e236ff1fa6b6721c6153eaddb69c23a5c51f532247b19", "str_len(trim(title)) > 0 && priority >= 1 && priority <= 4", ov_sb_finish(ovsb)->data);
  }
  bool t8 = (p5 == INT64_C(0));
  ov_str* t9 = 0; (void)t9;
  if (t8) {
    ov_str* t10 = ov_p_null_a6460182_0d7cc796();
    t9 = t10;
  } else {
    ov_str* t11 = ov_p_int_4640db5b_f21961d6(p5);
    t9 = t11;
  }
  ov_str* v1911 = t9;
  ov_str* t12 = ov_str_new("insert into issues (project_id, number, title, body, priority, assignee_id) ", 76);
  ov_str* t13 = ov_str_new("select \?, coalesce(max(number), 0) + 1, \?, \?, \?, \? from issues where project_id = \? returning id", 96);
  ov_str* t14 = ov_str_concat(t12, t13);
  ov_str* v1922 = t14;
  ov_str* t15 = ov_p_int_4640db5b_f21961d6(p1);
  ov_str* t16 = ov_p_str_73f85a82_17f92fe7(p2);
  ov_str* t17 = ov_p_str_73f85a82_17f92fe7(p3);
  ov_str* t18 = ov_p_int_4640db5b_f21961d6(p4);
  ov_str* t19 = ov_p_int_4640db5b_f21961d6(p1);
  ov_list* t20 = ov_list_new(6);
  t20->items[0] = (ov_value){.p = t15};
  t20->items[1] = (ov_value){.p = t16};
  t20->items[2] = (ov_value){.p = t17};
  t20->items[3] = (ov_value){.p = t18};
  t20->items[4] = (ov_value){.p = v1911};
  t20->items[5] = (ov_value){.p = t19};
  int64_t t21 = ov_db_int_36007c3c_9a9bc110(p0, v1922, t20);
  int64_t v1926 = t21;
  bool t22 = (v1926 < INT64_C(1));
  if (t22) {
    int64_t t23 = ov_neg(INT64_C(1), "create_issue", "c9275d15243ee94ce20e236ff1fa6b6721c6153eaddb69c23a5c51f532247b19");
    return t23;
  }
  ov_list* t24 = p6;
  for (int64_t t25 = 0; t25 < t24->len; t25++) {
    int64_t t26 = (t24->items[t25]).i;
    int64_t v1954 = t26;
    ov_str* t28 = ov_str_new("insert or ignore into issue_labels (issue_id, label_id) values (\?, \?)", 69);
    ov_str* t29 = ov_p_int_4640db5b_f21961d6(v1926);
    ov_str* t30 = ov_p_int_4640db5b_f21961d6(v1954);
    ov_list* t31 = ov_list_new(2);
    t31->items[0] = (ov_value){.p = t29};
    t31->items[1] = (ov_value){.p = t30};
    int64_t t32 = ov_db_exec_3d590b5f_20515640(p0, t28, t31);
  }
  ov_str* t33 = ov_str_new("created", 7);
  ov_str* t34 = ov_str_new("", 0);
  int64_t t35 = ov_log_activity_e9ca7bb3_dab08e1e(p0, v1926, t33, t34);
  ov_str* t36 = ov_str_new("select number from issues where id = \?", 38);
  ov_str* t37 = ov_p_int_4640db5b_f21961d6(v1926);
  ov_list* t38 = ov_list_new(1);
  t38->items[0] = (ov_value){.p = t37};
  int64_t t39 = ov_db_int_36007c3c_9a9bc110(p0, t36, t38);
  return t39;
}
