#include "ov_types.h"

int64_t ov_db_open_1ac4b90f_bff7d5eb(ov_str*);
int64_t ov_sqlite_exec_9e11f74e_df21757d(int64_t, ov_str*);
ov_list* ov_db_rows_2c7fa5b5_89b40a99(int64_t, ov_str*, ov_list*);
ov_str* ov_schema_sql_ec81cf05_68aabf13(void);
ovt_Option_6133f15b* ov_sqlite_query_p_9c03480a_2da2338d(int64_t, ov_str*, ov_str*);
uint8_t ov_sqlite_close_00467aec_ba12536b(int64_t);
uint8_t ov_test_t_db_schema_68953109(void);

uint8_t ov_test_t_db_schema_68953109(void) {
  ov_str* t1 = ov_str_new(":memory:", 8);
  int64_t t2 = ov_db_open_1ac4b90f_bff7d5eb(t1);
  int64_t v1326 = t2;
  ov_str* t3 = ov_str_new("select name from sqlite_master where type = 'table' and name != 'sqlite_sequence' order by name", 95);
  ov_list* t4 = ov_list_empty();
  ov_list* t5 = ov_db_rows_2c7fa5b5_89b40a99(v1326, t3, t4);
  ov_str* t6 = ov_str_new("activity", 8);
  ov_list* t7 = ov_list_new(1);
  t7->items[0] = (ov_value){.p = t6};
  ov_str* t8 = ov_str_new("comments", 8);
  ov_list* t9 = ov_list_new(1);
  t9->items[0] = (ov_value){.p = t8};
  ov_str* t10 = ov_str_new("issue_labels", 12);
  ov_list* t11 = ov_list_new(1);
  t11->items[0] = (ov_value){.p = t10};
  ov_str* t12 = ov_str_new("issues", 6);
  ov_list* t13 = ov_list_new(1);
  t13->items[0] = (ov_value){.p = t12};
  ov_str* t14 = ov_str_new("labels", 6);
  ov_list* t15 = ov_list_new(1);
  t15->items[0] = (ov_value){.p = t14};
  ov_str* t16 = ov_str_new("projects", 8);
  ov_list* t17 = ov_list_new(1);
  t17->items[0] = (ov_value){.p = t16};
  ov_str* t18 = ov_str_new("users", 5);
  ov_list* t19 = ov_list_new(1);
  t19->items[0] = (ov_value){.p = t18};
  ov_list* t20 = ov_list_new(7);
  t20->items[0] = (ov_value){.p = t7};
  t20->items[1] = (ov_value){.p = t9};
  t20->items[2] = (ov_value){.p = t11};
  t20->items[3] = (ov_value){.p = t13};
  t20->items[4] = (ov_value){.p = t15};
  t20->items[5] = (ov_value){.p = t17};
  t20->items[6] = (ov_value){.p = t19};
  if (!oveq_List_b24373fb(t5, t20)) ovsup_assert_eq_fail("t_db_schema", "68953109c9c8f05873f22bae1484a4fb9da1658987a3cb1099721b57bdce5c86", ovshow_List_b24373fb(t5), ovshow_List_b24373fb(t20));
  ov_str* t21 = ov_schema_sql_ec81cf05_68aabf13();
  int64_t t22 = ov_sqlite_exec_9e11f74e_df21757d(v1326, t21);
  if (!(t22 == INT64_C(0))) ovsup_assert_eq_fail("t_db_schema", "68953109c9c8f05873f22bae1484a4fb9da1658987a3cb1099721b57bdce5c86", ov_show_int(t22), ov_show_int(INT64_C(0)));
  ov_str* t23 = ov_str_new("select 1; select 2", 18);
  ov_str* t24 = ov_str_new("", 0);
  ovt_Option_6133f15b* t25 = ov_sqlite_query_p_9c03480a_2da2338d(v1326, t23, t24);
  ov_str* t26 = ov_str_new("1\0361", 3);
  ovt_Option_6133f15b* t27 = ovt_Option_6133f15b_mk0(t26);
  if (!oveq_Option_6133f15b(t25, t27)) ovsup_assert_eq_fail("t_db_schema", "68953109c9c8f05873f22bae1484a4fb9da1658987a3cb1099721b57bdce5c86", ovshow_Option_6133f15b(t25), ovshow_Option_6133f15b(t27));
  (void)(ov_sqlite_close_00467aec_ba12536b(v1326));
  return 0;
}
