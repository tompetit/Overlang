#include "ov_types.h"

ovt_Option_6133f15b* ov_last_3b93ed9c_16222bc9(ov_list* p0);

ovt_Option_6133f15b* ov_last_3b93ed9c_16222bc9(ov_list* p0) {
  int64_t t1 = p0->len;
  int64_t t2 = ov_sub(t1, INT64_C(1), "last", "3b93ed9c47a4dbea6fb3dc4d9f20fa32d850072f5b1ab86b09bc567e754d329a");
  ov_value t3 = {0};
  ovt_Option_6133f15b* t4 = ((ov_list_try_get(p0, t2, &t3)) ? ovt_Option_6133f15b_mk0(((ov_str*)(t3).p)) : ovt_Option_6133f15b_mk1());
  return t4;
}
