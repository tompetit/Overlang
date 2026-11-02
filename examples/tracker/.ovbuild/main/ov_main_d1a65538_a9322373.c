#include "ov_types.h"

int64_t ov_db_open_1ac4b90f_bff7d5eb(ov_str*);
int64_t ov_http_listen_20c64c9f_ec179ee1(int64_t);
ovt_tup2_7b8729bb* ov_parse_args_03e41b86_829441a9(ov_list*);
uint8_t ov_serve_5fc523c8_347af594(int64_t, int64_t);
uint8_t ov_main_d1a65538_a9322373(void);

uint8_t ov_main_d1a65538_a9322373(void) {
  ov_list* t1 = ov_args();
  ovt_tup2_7b8729bb* t2 = ov_parse_args_03e41b86_829441a9(t1);
  ov_str* v6359 = t2->f0;
  int64_t v6360 = t2->f1;
  int64_t t3 = ov_db_open_1ac4b90f_bff7d5eb(v6359);
  int64_t v6366 = t3;
  bool t4 = (v6366 < INT64_C(1));
  if (t4) {
    ov_str* t5 = ov_str_new("cannot open ", 12);
    ov_str* t6 = ov_str_concat(t5, v6359);
    ov_println(t6);
    ov_exit(INT64_C(1));
    return 0;
  }
  int64_t t7 = ov_http_listen_20c64c9f_ec179ee1(v6360);
  int64_t v6383 = t7;
  bool t8 = (v6383 < INT64_C(1));
  if (t8) {
    ov_str* t9 = ov_str_new("cannot listen on port ", 22);
    ov_str* t10 = ov_show_int(v6360);
    ov_str* t11 = ov_str_concat(t9, t10);
    ov_println(t11);
    ov_exit(INT64_C(1));
    return 0;
  }
  ov_str* t12 = ov_str_new("listening on http://127.0.0.1:", 30);
  ov_str* t13 = ov_show_int(v6360);
  ov_str* t14 = ov_str_concat(t12, t13);
  ov_println(t14);
  (void)(ov_flush());
  (void)(ov_serve_5fc523c8_347af594(v6366, v6383));
  return 0;
}
