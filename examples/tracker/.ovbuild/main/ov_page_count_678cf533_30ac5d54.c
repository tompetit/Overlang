#include "ov_types.h"

int64_t ov_max_c0a98029_5b0fcca4(int64_t, int64_t);
int64_t ov_page_count_678cf533_30ac5d54(int64_t p0);

int64_t ov_page_count_678cf533_30ac5d54(int64_t p0) {
  bool t1 = (p0 >= INT64_C(0));
  bool t2 = t1;
  if (t2) {
    bool t3 = (p0 <= INT64_C(1000000000));
    t2 = t3;
  }
  if (!t2) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"total\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "page_count", "678cf533ba65e0c1b9068e07b7e1f0626908ef9883f282fd98349e830c870873", "total >= 0 && total <= 1000000000", ov_sb_finish(ovsb)->data);
  }
  int64_t ovres = 0;
  int64_t t4 = ov_add(p0, INT64_C(19), "page_count", "678cf533ba65e0c1b9068e07b7e1f0626908ef9883f282fd98349e830c870873");
  int64_t t5 = ov_div(t4, INT64_C(20), "page_count", "678cf533ba65e0c1b9068e07b7e1f0626908ef9883f282fd98349e830c870873");
  int64_t t6 = ov_max_c0a98029_5b0fcca4(INT64_C(1), t5);
  ovres = t6;
  bool t7 = (ovres >= INT64_C(1));
  if (!t7) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"total\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"result\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(ovres)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("ensures", "page_count", "678cf533ba65e0c1b9068e07b7e1f0626908ef9883f282fd98349e830c870873", "result >= 1", ov_sb_finish(ovsb)->data);
  }
  return ovres;
}
