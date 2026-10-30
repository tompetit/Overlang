#include "ov_types.h"
#include "ov_http.h"

int64_t ov_http_respond_4cc639fd_f73d4847(int64_t p0, int64_t p1, ov_str* p2, ov_str* p3);

int64_t ov_http_respond_4cc639fd_f73d4847(int64_t p0, int64_t p1, ov_str* p2, ov_str* p3) {
  return ov_http_respond(p0, p1, p2->data, p3->data);
}
