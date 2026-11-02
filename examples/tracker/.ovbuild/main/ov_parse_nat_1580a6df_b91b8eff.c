#include "ov_types.h"

bool ov_all_ba8a189d_8d783bfe(ov_list*, ov_closure*);
bool ov_is_digit_d7695ac0_63fa906b(ov_str*);
int64_t ov_unwrap_or_e7341bb9_bde9d67c(ovt_Option_d5f2c30a*, int64_t);
int64_t ov_parse_nat_1580a6df_b91b8eff(ov_str* p0);

static bool ovl_ov_parse_nat_1580a6df_b91b8eff_4903(void* ovenv, ov_str* l4903_0) {
  bool t7 = ov_is_digit_d7695ac0_63fa906b(l4903_0);
  return t7;
}

int64_t ov_parse_nat_1580a6df_b91b8eff(ov_str* p0) {
  int64_t ovres = 0;
  int64_t t1 = ov_str_len_cps(p0);
  int64_t v4885 = t1;
  bool t2 = (v4885 < INT64_C(1));
  bool t3 = t2;
  if (!t3) {
    bool t4 = (v4885 > INT64_C(9));
    t3 = t4;
  }
  bool t5 = t3;
  if (!t5) {
    ov_list* t6 = ov_str_chars(p0);
    ov_closure* t8 = ov_closure_mk((void*)ovl_ov_parse_nat_1580a6df_b91b8eff_4903, NULL);
    bool t9 = ov_all_ba8a189d_8d783bfe(t6, t8);
    bool t10 = (!t9);
    t5 = t10;
  }
  int64_t t11 = 0; (void)t11;
  if (t5) {
    int64_t t12 = ov_neg(INT64_C(1), "parse_nat", "1580a6df82bbc079241f7d3bd51e624b398cad28fcd0b207fb0bb423104a1618");
    t11 = t12;
  } else {
    int64_t t13 = 0;
    ovt_Option_d5f2c30a* t14 = ((ov_parse_int(p0, &t13)) ? ovt_Option_d5f2c30a_mk0(t13) : ovt_Option_d5f2c30a_mk1());
    int64_t t15 = ov_neg(INT64_C(1), "parse_nat", "1580a6df82bbc079241f7d3bd51e624b398cad28fcd0b207fb0bb423104a1618");
    int64_t t16 = ov_unwrap_or_e7341bb9_bde9d67c(t14, t15);
    t11 = t16;
  }
  ovres = t11;
  int64_t t17 = ov_neg(INT64_C(1), "parse_nat", "1580a6df82bbc079241f7d3bd51e624b398cad28fcd0b207fb0bb423104a1618");
  bool t18 = (ovres >= t17);
  if (!t18) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"s\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p0)));
    ov_sb_push_cstr(ovsb, ", \"result\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(ovres)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("ensures", "parse_nat", "1580a6df82bbc079241f7d3bd51e624b398cad28fcd0b207fb0bb423104a1618", "result >= -1", ov_sb_finish(ovsb)->data);
  }
  return ovres;
}
