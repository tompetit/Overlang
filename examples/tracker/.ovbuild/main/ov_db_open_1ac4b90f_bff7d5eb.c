#include "ov_types.h"

int64_t ov_sqlite_exec_9e11f74e_df21757d(int64_t, ov_str*);
int64_t ov_sqlite_open_5ed5ead5_1cd01d01(ov_str*);
ov_str* ov_schema_sql_ec81cf05_68aabf13(void);
uint8_t ov_sqlite_close_00467aec_ba12536b(int64_t);
int64_t ov_db_open_1ac4b90f_bff7d5eb(ov_str* p0);

int64_t ov_db_open_1ac4b90f_bff7d5eb(ov_str* p0) {
  int64_t t1 = ov_sqlite_open_5ed5ead5_1cd01d01(p0);
  int64_t v1049 = t1;
  bool t2 = (v1049 < INT64_C(1));
  int64_t t3 = 0; (void)t3;
  if (t2) {
    int64_t t4 = ov_neg(INT64_C(1), "db_open", "1ac4b90fa5d0a31f3bfaf369ad97f4c903f6d632d446fd5223d692bc988b20d5");
    t3 = t4;
  } else {
    ov_str* t5 = ov_schema_sql_ec81cf05_68aabf13();
    int64_t t6 = ov_sqlite_exec_9e11f74e_df21757d(v1049, t5);
    bool t7 = (t6 < INT64_C(0));
    int64_t t8 = 0; (void)t8;
    if (t7) {
      (void)(ov_sqlite_close_00467aec_ba12536b(v1049));
      int64_t t9 = ov_neg(INT64_C(1), "db_open", "1ac4b90fa5d0a31f3bfaf369ad97f4c903f6d632d446fd5223d692bc988b20d5");
      t8 = t9;
    } else {
      t8 = v1049;
    }
    t3 = t8;
  }
  return t3;
}
