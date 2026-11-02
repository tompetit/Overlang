#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_form_value_a2c19651_af36f5f9(ov_str* p0, ov_str* p1);

ov_str* ov_form_value_a2c19651_af36f5f9(ov_str* p0, ov_str* p1) {
  const char* r = ov_http_form_value(p0->data, p1->data);
  if (!r) ov_trap("foreign_null", "form_value", "a2c1965190c4ae022308022bcbea627ed6d0c893d81880df30c9c8f9338518af", "\"`ov_http_form_value` returned NULL\"");
  return ov_str_lit(r);
}
