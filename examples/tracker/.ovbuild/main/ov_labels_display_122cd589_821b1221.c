#include "ov_types.h"

ov_str* ov_dash_if_empty_4d9d1960_b97c4fb0(ov_str*);
ov_str* ov_labels_display_122cd589_821b1221(ov_list* p0);

ov_str* ov_labels_display_122cd589_821b1221(ov_list* p0) {
  ov_str* t1 = ov_str_new(", ", 2);
  ov_str* t2 = ov_str_join(p0, t1);
  ov_str* t3 = ov_dash_if_empty_4d9d1960_b97c4fb0(t2);
  return t3;
}
