#include "ov_types.h"

int64_t ov_parse_nat_1580a6df_b91b8eff(ov_str*);
int64_t ov_issue_ref_3f7d9f9a_ccc011e4(ov_str* p0, ov_str* p1);

int64_t ov_issue_ref_3f7d9f9a_ccc011e4(ov_str* p0, ov_str* p1) {
  int64_t ovres = 0;
  bool t1 = ov_str_ends_with(p0, p1);
  bool t2 = (!t1);
  int64_t t3 = 0; (void)t3;
  if (t2) {
    t3 = INT64_C(0);
  } else {
    int64_t t4 = ov_str_len_cps(p0);
    int64_t t5 = ov_str_len_cps(p1);
    int64_t t6 = ov_sub(t4, t5, "issue_ref", "3f7d9f9aa7e060a92efaa43f0df1877d0638e116bee0572c6ed72373e664b387");
    ov_str* t7 = ov_str_substr(p0, INT64_C(0), t6);
    int64_t t8 = ov_parse_nat_1580a6df_b91b8eff(t7);
    int64_t v4927 = t8;
    bool t9 = (v4927 >= INT64_C(1));
    int64_t t10 = 0; (void)t10;
    if (t9) {
      t10 = v4927;
    } else {
      t10 = INT64_C(0);
    }
    t3 = t10;
  }
  ovres = t3;
  bool t11 = (ovres >= INT64_C(0));
  if (!t11) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"seg\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p0)));
    ov_sb_push_cstr(ovsb, ", \"suffix\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p1)));
    ov_sb_push_cstr(ovsb, ", \"result\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(ovres)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("ensures", "issue_ref", "3f7d9f9aa7e060a92efaa43f0df1877d0638e116bee0572c6ed72373e664b387", "result >= 0", ov_sb_finish(ovsb)->data);
  }
  return ovres;
}
