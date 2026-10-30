#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_html_escape_bb603171_d5a1eb27(ov_str* p0);

ov_str* ov_html_escape_bb603171_d5a1eb27(ov_str* p0) {
  const char* r = ov_http_html_escape(p0->data);
  if (!r) ov_trap("foreign_null", "html_escape", "", "\"`ov_http_html_escape` returned NULL\"");
  return ov_str_lit(r);
}
