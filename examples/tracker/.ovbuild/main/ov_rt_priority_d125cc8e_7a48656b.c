#include "ov_types.h"

int64_t ov_parse_nat_1580a6df_b91b8eff(ov_str*);
int64_t ov_rt_priority_d125cc8e_7a48656b(ov_str* p0, int64_t p1);

int64_t ov_rt_priority_d125cc8e_7a48656b(ov_str* p0, int64_t p1) {
  ov_str* t1 = ov_str_new("", 0);
  bool t2 = ov_str_eq(p0, t1);
  int64_t t3 = 0; (void)t3;
  if (t2) {
    t3 = p1;
  } else {
    int64_t t4 = ov_parse_nat_1580a6df_b91b8eff(p0);
    t3 = t4;
  }
  return t3;
}
