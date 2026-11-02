#include "ov_types.h"

int64_t ov_create_label_da024719_33b12a1b(int64_t, int64_t, ov_str*);
int64_t ov_create_project_2373b13f_51d6e4b6(int64_t, ov_str*, ov_str*);
int64_t ov_create_user_f316dd39_339cb8a1(int64_t, ov_str*);
uint8_t ov_m_fixture_6ba66877_59f8675e(int64_t p0);

uint8_t ov_m_fixture_6ba66877_59f8675e(int64_t p0) {
  ov_str* t1 = ov_str_new("alice", 5);
  int64_t t2 = ov_create_user_f316dd39_339cb8a1(p0, t1);
  if (!(t2 == INT64_C(1))) ovsup_assert_eq_fail("m_fixture", "6ba66877f19bceecd6a92416951a2e895fb800743979b9bbd89457210b69051f", ov_show_int(t2), ov_show_int(INT64_C(1)));
  ov_str* t3 = ov_str_new("bob", 3);
  int64_t t4 = ov_create_user_f316dd39_339cb8a1(p0, t3);
  if (!(t4 == INT64_C(2))) ovsup_assert_eq_fail("m_fixture", "6ba66877f19bceecd6a92416951a2e895fb800743979b9bbd89457210b69051f", ov_show_int(t4), ov_show_int(INT64_C(2)));
  ov_str* t5 = ov_str_new("WEB", 3);
  ov_str* t6 = ov_str_new("Web site", 8);
  int64_t t7 = ov_create_project_2373b13f_51d6e4b6(p0, t5, t6);
  if (!(t7 == INT64_C(1))) ovsup_assert_eq_fail("m_fixture", "6ba66877f19bceecd6a92416951a2e895fb800743979b9bbd89457210b69051f", ov_show_int(t7), ov_show_int(INT64_C(1)));
  ov_str* t8 = ov_str_new("bug", 3);
  int64_t t9 = ov_create_label_da024719_33b12a1b(p0, INT64_C(1), t8);
  if (!(t9 == INT64_C(1))) ovsup_assert_eq_fail("m_fixture", "6ba66877f19bceecd6a92416951a2e895fb800743979b9bbd89457210b69051f", ov_show_int(t9), ov_show_int(INT64_C(1)));
  ov_str* t10 = ov_str_new("ui", 2);
  int64_t t11 = ov_create_label_da024719_33b12a1b(p0, INT64_C(1), t10);
  if (!(t11 == INT64_C(2))) ovsup_assert_eq_fail("m_fixture", "6ba66877f19bceecd6a92416951a2e895fb800743979b9bbd89457210b69051f", ov_show_int(t11), ov_show_int(INT64_C(2)));
  return 0;
}
