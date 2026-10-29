#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_form_value_45246b5c_af36f5f9(ov_str* p0, ov_str* p1);

ov_str* ov_form_value_45246b5c_af36f5f9(ov_str* p0, ov_str* p1) {
  const char* r = ov_http_form_value(p0->data, p1->data);
  if (!r) ov_trap("foreign_null", "form_value", "", "\"`ov_http_form_value` returned NULL\"");
  return ov_str_lit(r);
}
