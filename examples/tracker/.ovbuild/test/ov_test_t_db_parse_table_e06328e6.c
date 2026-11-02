#include "ov_types.h"

int64_t ov_cell_int_241695c3_c8b376c8(ov_list*, int64_t);
ov_list* ov_parse_table_70d80d96_bf8ac7fa(ov_str*);
ov_str* ov_cell_str_c5e3a532_bb12c045(ov_list*, int64_t);
uint8_t ov_test_t_db_parse_table_e06328e6(void);

uint8_t ov_test_t_db_parse_table_e06328e6(void) {
  ov_str* t1 = ov_str_new("a\037b\0361\037x\0362\037", 10);
  ov_list* t2 = ov_parse_table_70d80d96_bf8ac7fa(t1);
  ov_str* t3 = ov_str_new("1", 1);
  ov_str* t4 = ov_str_new("x", 1);
  ov_list* t5 = ov_list_new(2);
  t5->items[0] = (ov_value){.p = t3};
  t5->items[1] = (ov_value){.p = t4};
  ov_str* t6 = ov_str_new("2", 1);
  ov_str* t7 = ov_str_new("", 0);
  ov_list* t8 = ov_list_new(2);
  t8->items[0] = (ov_value){.p = t6};
  t8->items[1] = (ov_value){.p = t7};
  ov_list* t9 = ov_list_new(2);
  t9->items[0] = (ov_value){.p = t5};
  t9->items[1] = (ov_value){.p = t8};
  if (!oveq_List_b24373fb(t2, t9)) ovsup_assert_eq_fail("t_db_parse_table", "e06328e6fe8ca79801b454514f384819e5c88d2a8f70937d3ae47c78c02c0b74", ovshow_List_b24373fb(t2), ovshow_List_b24373fb(t9));
  ov_str* t10 = ov_str_new("a\037b", 3);
  ov_list* t11 = ov_parse_table_70d80d96_bf8ac7fa(t10);
  ov_list* t12 = ov_list_empty();
  if (!oveq_List_b24373fb(t11, t12)) ovsup_assert_eq_fail("t_db_parse_table", "e06328e6fe8ca79801b454514f384819e5c88d2a8f70937d3ae47c78c02c0b74", ovshow_List_b24373fb(t11), ovshow_List_b24373fb(t12));
  ov_str* t13 = ov_str_new("", 0);
  ov_list* t14 = ov_parse_table_70d80d96_bf8ac7fa(t13);
  ov_list* t15 = ov_list_empty();
  if (!oveq_List_b24373fb(t14, t15)) ovsup_assert_eq_fail("t_db_parse_table", "e06328e6fe8ca79801b454514f384819e5c88d2a8f70937d3ae47c78c02c0b74", ovshow_List_b24373fb(t14), ovshow_List_b24373fb(t15));
  ov_str* t16 = ov_str_new("a", 1);
  ov_list* t17 = ov_list_new(1);
  t17->items[0] = (ov_value){.p = t16};
  ov_str* t18 = ov_cell_str_c5e3a532_bb12c045(t17, INT64_C(3));
  ov_str* t19 = ov_str_new("", 0);
  if (!ov_str_eq(t18, t19)) ovsup_assert_eq_fail("t_db_parse_table", "e06328e6fe8ca79801b454514f384819e5c88d2a8f70937d3ae47c78c02c0b74", ov_show_str(t18), ov_show_str(t19));
  ov_str* t20 = ov_str_new("7", 1);
  ov_str* t21 = ov_str_new("x", 1);
  ov_list* t22 = ov_list_new(2);
  t22->items[0] = (ov_value){.p = t20};
  t22->items[1] = (ov_value){.p = t21};
  int64_t t23 = ov_cell_int_241695c3_c8b376c8(t22, INT64_C(0));
  if (!(t23 == INT64_C(7))) ovsup_assert_eq_fail("t_db_parse_table", "e06328e6fe8ca79801b454514f384819e5c88d2a8f70937d3ae47c78c02c0b74", ov_show_int(t23), ov_show_int(INT64_C(7)));
  ov_str* t24 = ov_str_new("7", 1);
  ov_str* t25 = ov_str_new("x", 1);
  ov_list* t26 = ov_list_new(2);
  t26->items[0] = (ov_value){.p = t24};
  t26->items[1] = (ov_value){.p = t25};
  int64_t t27 = ov_cell_int_241695c3_c8b376c8(t26, INT64_C(1));
  if (!(t27 == INT64_C(0))) ovsup_assert_eq_fail("t_db_parse_table", "e06328e6fe8ca79801b454514f384819e5c88d2a8f70937d3ae47c78c02c0b74", ov_show_int(t27), ov_show_int(INT64_C(0)));
  return 0;
}
