#include "ov_types.h"

int64_t ov_db_init_4a9b1b67_8228de64(int64_t);
int64_t ov_http_listen_40f85aa5_ec179ee1(int64_t);
int64_t ov_port_from_e599bbcb_aa28173d(ov_list*);
int64_t ov_sqlite_open_684da90e_1cd01d01(ov_str*);
ov_str* ov_sqlite_error_7a6396d5_605f1880(int64_t);
uint8_t ov_serve_21e06530_347af594(int64_t, int64_t);
uint8_t ov_main_1f5962a2_a9322373(void);

uint8_t ov_main_1f5962a2_a9322373(void) {
  ov_list* t1 = ov_args();
  int64_t t2 = ov_port_from_e599bbcb_aa28173d(t1);
  int64_t v702 = t2;
  ov_str* t3 = ov_str_new("todo.db", 7);
  int64_t t4 = ov_sqlite_open_684da90e_1cd01d01(t3);
  int64_t v707 = t4;
  int64_t t5 = ov_http_listen_40f85aa5_ec179ee1(v702);
  int64_t v711 = t5;
  bool t6 = (v707 < INT64_C(1));
  if (t6) {
    ov_str* t7 = ov_str_new("cannot open todo.db: ", 21);
    ov_str* t8 = ov_sqlite_error_7a6396d5_605f1880(INT64_C(0));
    ov_str* t9 = ov_str_concat(t7, t8);
    ov_println(t9);
  } else {
    bool t10 = (v711 < INT64_C(1));
    if (t10) {
      ov_str* t11 = ov_str_new("cannot listen on port ", 22);
      ov_str* t12 = ov_show_int(v702);
      ov_str* t13 = ov_str_concat(t11, t12);
      ov_println(t13);
    } else {
      int64_t t14 = ov_db_init_4a9b1b67_8228de64(v707);
      ov_str* t15 = ov_str_new("listening on http://127.0.0.1:", 30);
      ov_str* t16 = ov_show_int(v702);
      ov_str* t17 = ov_str_concat(t15, t16);
      ov_println(t17);
      (void)(ov_serve_21e06530_347af594(v707, v711));
    }
  }
  return 0;
}
