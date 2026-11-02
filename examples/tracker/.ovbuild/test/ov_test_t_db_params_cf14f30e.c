#include "ov_types.h"

ov_str* ov_p_int_4640db5b_f21961d6(int64_t);
ov_str* ov_p_null_a6460182_0d7cc796(void);
ov_str* ov_p_str_73f85a82_17f92fe7(ov_str*);
ov_str* ov_sql_params_f88159f9_4e330225(ov_list*);
uint8_t ov_test_t_db_params_cf14f30e(void);

uint8_t ov_test_t_db_params_cf14f30e(void) {
  ov_str* t1 = ov_p_int_4640db5b_f21961d6(INT64_C(5));
  ov_str* t2 = ov_str_new("a b", 3);
  ov_str* t3 = ov_p_str_73f85a82_17f92fe7(t2);
  ov_str* t4 = ov_p_null_a6460182_0d7cc796();
  int64_t t5 = ov_neg(INT64_C(2), "t_db_params", "cf14f30e9efb0e077a2fee78e4658d406ee7d52c88194c192d969a0ea121a5af");
  ov_str* t6 = ov_p_int_4640db5b_f21961d6(t5);
  ov_list* t7 = ov_list_new(4);
  t7->items[0] = (ov_value){.p = t1};
  t7->items[1] = (ov_value){.p = t3};
  t7->items[2] = (ov_value){.p = t4};
  t7->items[3] = (ov_value){.p = t6};
  ov_str* t8 = ov_sql_params_f88159f9_4e330225(t7);
  ov_str* t9 = ov_str_new("i5\037sa b\037n\037i-2", 13);
  if (!ov_str_eq(t8, t9)) ovsup_assert_eq_fail("t_db_params", "cf14f30e9efb0e077a2fee78e4658d406ee7d52c88194c192d969a0ea121a5af", ov_show_str(t8), ov_show_str(t9));
  ov_list* t10 = ov_list_empty();
  ov_str* t11 = ov_sql_params_f88159f9_4e330225(t10);
  ov_str* t12 = ov_str_new("", 0);
  if (!ov_str_eq(t11, t12)) ovsup_assert_eq_fail("t_db_params", "cf14f30e9efb0e077a2fee78e4658d406ee7d52c88194c192d969a0ea121a5af", ov_show_str(t11), ov_show_str(t12));
  return 0;
}
