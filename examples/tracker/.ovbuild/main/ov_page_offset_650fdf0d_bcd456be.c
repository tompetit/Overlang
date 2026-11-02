#include "ov_types.h"

int64_t ov_page_offset_650fdf0d_bcd456be(int64_t p0);

int64_t ov_page_offset_650fdf0d_bcd456be(int64_t p0) {
  bool t1 = (p0 >= INT64_C(1));
  bool t2 = t1;
  if (t2) {
    bool t3 = (p0 <= INT64_C(1000000));
    t2 = t3;
  }
  if (!t2) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"page\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("requires", "page_offset", "650fdf0de2a096146995366a8fbd5cf2953dabe35ac5fbfc0202b598324bba70", "page >= 1 && page <= 1000000", ov_sb_finish(ovsb)->data);
  }
  int64_t ovres = 0;
  int64_t t4 = ov_sub(p0, INT64_C(1), "page_offset", "650fdf0de2a096146995366a8fbd5cf2953dabe35ac5fbfc0202b598324bba70");
  int64_t t5 = ov_mul(t4, INT64_C(20), "page_offset", "650fdf0de2a096146995366a8fbd5cf2953dabe35ac5fbfc0202b598324bba70");
  ovres = t5;
  bool t6 = (ovres >= INT64_C(0));
  if (!t6) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"page\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(p0)));
    ov_sb_push_cstr(ovsb, ", \"result\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_int(ovres)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("ensures", "page_offset", "650fdf0de2a096146995366a8fbd5cf2953dabe35ac5fbfc0202b598324bba70", "result >= 0", ov_sb_finish(ovsb)->data);
  }
  return ovres;
}
