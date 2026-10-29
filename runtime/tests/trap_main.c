/* Tiny executable that provokes one trap, chosen by argv[1]; the Makefile checks exit code 3 and stderr. */
#include "../rt.h"

#include <string.h>

int main(int argc, char** argv) {
  ov_init(argc, argv);
  ov_println(ov_str_lit("before"));
  const char* k = argv[1];
  if (!strcmp(k, "add")) ov_add(INT64_MAX, 1, "f", "abc");
  if (!strcmp(k, "sub")) ov_sub(INT64_MIN, 1, "f", "abc");
  if (!strcmp(k, "mul")) ov_mul(INT64_MAX, 2, "f", "abc");
  if (!strcmp(k, "neg")) ov_neg(INT64_MIN, "f", "abc");
  if (!strcmp(k, "div0")) ov_div(1, 0, "f", "abc");
  if (!strcmp(k, "mod0")) ov_mod(1, 0, "f", "abc");
  if (!strcmp(k, "divmin")) ov_div(INT64_MIN, -1, "f", "abc");
  if (!strcmp(k, "bounds")) ov_list_get(ov_list_empty(), 2, "f", "abc");
  if (!strcmp(k, "plain")) ov_trap("assert", "f", "abc", NULL);
  if (!strcmp(k, "esc")) ov_trap("todo", "f\"x", "abc", "{\"a\": 1}");
  if (!strcmp(k, "requires")) ov_trap_contract("requires", "f", "abc", "x > 0", "{\"x\": \"-1\"}");
  if (!strcmp(k, "ensures")) ov_trap_contract("ensures", "f", "abc", "result >= 0", NULL);
  return 0;
}
