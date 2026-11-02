#include "ov_types.h"
#include "ov_http.h"

ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str* p0);

ov_str* ov_html_escape_114cb911_d5a1eb27(ov_str* p0) {
  const char* r = ov_http_html_escape(p0->data);
  if (!r) ov_trap("foreign_null", "html_escape", "114cb911352e263627e9ec556f6a782a69257bf9d938929d3fecea5efa250bf3", "\"`ov_http_html_escape` returned NULL\"");
  return ov_str_lit(r);
}
