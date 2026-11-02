#include "ov_types.h"

ovt_Option_424abdc7* ov_last_3b93ed9c_20fe247f(ov_list* p0);

ovt_Option_424abdc7* ov_last_3b93ed9c_20fe247f(ov_list* p0) {
  int64_t t1 = p0->len;
  int64_t t2 = ov_sub(t1, INT64_C(1), "last", "3b93ed9c47a4dbea6fb3dc4d9f20fa32d850072f5b1ab86b09bc567e754d329a");
  ov_value t3 = {0};
  ovt_Option_424abdc7* t4 = ((ov_list_try_get(p0, t2, &t3)) ? ovt_Option_424abdc7_mk0(((ovt_Activity_dd4dbed0*)(t3).p)) : ovt_Option_424abdc7_mk1());
  return t4;
}
