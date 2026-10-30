#include "ov_types.h"

bool ov_all_e6f09b19_e9cb4669(ov_list*, ov_closure*);
ov_list* ov_parse_rows_e9206b62_50e2d628(ov_str* p0);

static bool ovl_ov_parse_rows_e9206b62_50e2d628_360(void* ovenv, ovt_Todo_6462003c* l360_0) {
  int64_t t21 = l360_0->f_id;
  bool t22 = (t21 > INT64_C(0));
  return t22;
}

ov_list* ov_parse_rows_e9206b62_50e2d628(ov_str* p0) {
  ov_list* ovres = 0;
  ov_list* t1 = ov_list_empty();
  ov_list* v362 = t1;
  (void)v362;
  ov_str* t2 = ov_str_new("\036", 1);
  ov_list* t3 = ov_str_split(p0, t2);
  ov_list* t4 = ov_list_drop(t3, INT64_C(1));
  ov_list* t5 = t4;
  for (int64_t t6 = 0; t6 < t5->len; t6++) {
    ov_str* t7 = ((ov_str*)(t5->items[t6]).p);
    ov_str* v364 = t7;
    ov_str* t8 = ov_str_new("\037", 1);
    ov_list* t9 = ov_str_split(v364, t8);
    bool t10 = false; (void)t10;
    if (!t10) {
      if ((t9->len == 3)) {
        ov_str* v376 = ((ov_str*)(t9->items[0]).p);
        ov_str* v377 = ((ov_str*)(t9->items[1]).p);
        ov_str* v378 = ((ov_str*)(t9->items[2]).p);
        t10 = true;
        int64_t t11 = 0;
        ovt_Option_d5f2c30a* t12 = ((ov_parse_int(v376, &t11)) ? ovt_Option_d5f2c30a_mk0(t11) : ovt_Option_d5f2c30a_mk1());
        bool t13 = false; (void)t13;
        if (!t13) {
          if ((t12->tag == 0)) {
            int64_t v383 = t12->u.c0.f0;
            bool t14 = (v383 > INT64_C(0));
            if (t14) {
              t13 = true;
              ov_list* t15 = v362;
              ov_str* t16 = ov_str_new("1", 1);
              bool t17 = ov_str_eq(v378, t16);
              ovt_Todo_6462003c* t18 = ovt_Todo_6462003c_mk(v383, v377, t17);
              ov_list* t19 = ov_list_push(t15, (ov_value){.p = t18});
              v362 = t19;
            }
          }
        }
        if (!t13) {
          if (true) {
            t13 = true;
          }
        }
      }
    }
    if (!t10) {
      if (true) {
        t10 = true;
      }
    }
  }
  ov_list* t20 = v362;
  ovres = t20;
  ov_closure* t23 = ov_closure_mk((void*)ovl_ov_parse_rows_e9206b62_50e2d628_360, NULL);
  bool t24 = ov_all_e6f09b19_e9cb4669(ovres, t23);
  if (!t24) {
    ov_sb* ovsb = ov_sb_new();
    ov_sb_push_cstr(ovsb, "{");
    ov_sb_push_cstr(ovsb, "\"text\": ");
    ov_sb_push(ovsb, ovsup_json_str(ov_show_str(p0)));
    ov_sb_push_cstr(ovsb, ", \"result\": ");
    ov_sb_push(ovsb, ovsup_json_str(ovshow_List_fa550908(ovres)));
    ov_sb_push_cstr(ovsb, "}");
    ov_trap_contract("ensures", "parse_rows", "", "all(result, |t| t.id > 0)", ov_sb_finish(ovsb)->data);
  }
  return ovres;
}
