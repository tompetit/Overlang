/* Runtime unit tests. Trap behaviour is covered by trap_main.c via the Makefile. */
#include "../rt.h"

#include <stdio.h>
#include <string.h>

static int checks, failures;
#define CHECK(c) do { checks++; if (!(c)) { failures++; fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); } } while (0)
#define S(x) ov_str_lit(x)
#define STR_IS(s, lit) (strcmp((s)->data, (lit)) == 0)
#define I(x) ((ov_value){.i = (x)})
#define P(x) ((ov_value){.p = (x)})

static ov_list* strs(int n, const char** xs) {
  ov_list* l = ov_list_new(n);
  for (int i = 0; i < n; i++) l->items[i].p = S(xs[i]);
  return l;
}
static bool list_is(ov_list* l, int n, const char** xs) {
  if (l->len != n) return false;
  for (int i = 0; i < n; i++) if (!STR_IS((ov_str*)l->items[i].p, xs[i])) return false;
  return true;
}

static void test_arith(void) {
  CHECK(ov_add(2, 3, "t", "h") == 5);
  CHECK(ov_sub(2, 3, "t", "h") == -1);
  CHECK(ov_mul(-4, 3, "t", "h") == -12);
  CHECK(ov_neg(5, "t", "h") == -5);
  CHECK(ov_div(-7, 2, "t", "h") == -3);
  CHECK(ov_mod(-7, 2, "t", "h") == -1);
  CHECK(ov_mod(7, -2, "t", "h") == 1);
  CHECK(ov_div_floor(-7, 2, "t", "h") == -4);
  CHECK(ov_div_floor(7, 2, "t", "h") == 3);
  CHECK(ov_div_floor(7, -2, "t", "h") == -4);
  CHECK(ov_div_floor(-6, 2, "t", "h") == -3);
  CHECK(ov_mod_floor(-7, 2, "t", "h") == 1);
  CHECK(ov_mod_floor(7, -2, "t", "h") == -1);
  CHECK(ov_mod_floor(6, 3, "t", "h") == 0);
  CHECK(ov_mod(INT64_MIN, -1, "t", "h") == 0);
  CHECK(ov_add(INT64_MAX, 0, "t", "h") == INT64_MAX);
}

static void test_utf8(void) {
  ov_str* s = S("h\xc3\xa9y");
  CHECK(s->len == 4);
  CHECK(s->cps == 3);
  CHECK(ov_str_len_cps(s) == 3);
  const char* want[] = {"h", "\xc3\xa9", "y"};
  CHECK(list_is(ov_str_chars(s), 3, want));
  CHECK(STR_IS(ov_str_substr(s, 1, 2), "\xc3\xa9"));
  CHECK(STR_IS(ov_str_substr(s, 1, 100), "\xc3\xa9y"));
  CHECK(STR_IS(ov_str_substr(s, -5, 1), "h"));
  CHECK(ov_str_substr(s, 2, 1)->len == 0);
  CHECK(STR_IS(ov_str_char_at(s, 1), "\xc3\xa9"));
  CHECK(ov_str_char_at(s, 3) == NULL);
  CHECK(ov_str_char_at(s, -1) == NULL);
  bool ok;
  CHECK(ov_str_code_point(S("\xc3\xa9"), &ok) == 0xE9 && ok);
  CHECK(ov_str_code_point(S("\xf0\x9f\x98\x80"), &ok) == 0x1F600 && ok);
  CHECK(ov_str_code_point(S("A"), &ok) == 65 && ok);
  ov_str_code_point(S(""), &ok);
  CHECK(!ok);
  CHECK(STR_IS(ov_str_from_code_point(0xE9), "\xc3\xa9"));
  CHECK(STR_IS(ov_str_from_code_point(0x20AC), "\xe2\x82\xac"));
  CHECK(STR_IS(ov_str_from_code_point(0x1F600), "\xf0\x9f\x98\x80"));
  CHECK(ov_str_from_code_point(0x1F600)->cps == 1);
  CHECK(STR_IS(ov_str_from_code_point(-1), "\xef\xbf\xbd"));
  CHECK(ov_str_concat(s, s)->cps == 6);
}

static void test_strings(void) {
  CHECK(ov_str_eq(S("ab"), S("ab")));
  CHECK(!ov_str_eq(S("ab"), S("abc")));
  CHECK(ov_str_cmp(S("a"), S("b")) < 0);
  CHECK(ov_str_cmp(S("b"), S("a")) > 0);
  CHECK(ov_str_cmp(S("a"), S("ab")) < 0);
  CHECK(ov_str_cmp(S("ab"), S("ab")) == 0);
  CHECK(STR_IS(ov_str_concat(S("foo"), S("bar")), "foobar"));
  CHECK(STR_IS(ov_str_new("abcdef", 3), "abc"));

  const char* w1[] = {"a", "b", "c"};
  CHECK(list_is(ov_str_split(S("a,b,c"), S(",")), 3, w1));
  const char* w2[] = {"a", "", "b", ""};
  CHECK(list_is(ov_str_split(S("a,,b,"), S(",")), 4, w2));
  const char* w3[] = {""};
  CHECK(list_is(ov_str_split(S(""), S(",")), 1, w3));
  const char* w4[] = {"", "x", ""};
  CHECK(list_is(ov_str_split(S("::x::"), S("::")), 3, w4));
  CHECK(list_is(ov_str_split(S("abc"), S("")), 3, w1));
  CHECK(ov_str_split(S(""), S(""))->len == 0);

  CHECK(STR_IS(ov_str_trim(S(" \t\n hi there \r\n")), "hi there"));
  CHECK(ov_str_trim(S("   "))->len == 0);
  CHECK(STR_IS(ov_str_to_upper(S("abC1\xc3\xa9")), "ABC1\xc3\xa9"));
  CHECK(STR_IS(ov_str_to_lower(S("ABc")), "abc"));
  CHECK(ov_str_starts_with(S("hello"), S("he")));
  CHECK(!ov_str_starts_with(S("he"), S("hello")));
  CHECK(ov_str_ends_with(S("hello"), S("llo")));
  CHECK(!ov_str_ends_with(S("lo"), S("llo")));
  CHECK(ov_str_contains(S("hello"), S("ell")));
  CHECK(!ov_str_contains(S("hello"), S("xyz")));
  CHECK(ov_str_contains(S("hello"), S("")));
  CHECK(STR_IS(ov_str_replace(S("aaaa"), S("aa"), S("b")), "bb"));
  CHECK(STR_IS(ov_str_replace(S("a-b-c"), S("-"), S("--")), "a--b--c"));
  CHECK(STR_IS(ov_str_replace(S("abc"), S("x"), S("y")), "abc"));
  CHECK(STR_IS(ov_str_replace(S("abc"), S("abc"), S("")), ""));
  CHECK(STR_IS(ov_str_replace(S("abc"), S(""), S("y")), "abc"));
  CHECK(STR_IS(ov_str_repeat(S("ab"), 3), "ababab"));
  CHECK(ov_str_repeat(S("ab"), 0)->len == 0);
  CHECK(ov_str_repeat(S("ab"), -2)->len == 0);
  CHECK(ov_str_repeat(S("\xc3\xa9"), 3)->cps == 3);
  CHECK(STR_IS(ov_str_join(strs(3, w1), S(", ")), "a, b, c"));
  CHECK(ov_str_join(ov_list_empty(), S(","))->len == 0);
}

static void test_parse_show(void) {
  int64_t n;
  double f;
  CHECK(ov_parse_int(S("42"), &n) && n == 42);
  CHECK(ov_parse_int(S("-42"), &n) && n == -42);
  CHECK(ov_parse_int(S("+7"), &n) && n == 7);
  CHECK(ov_parse_int(S("9223372036854775807"), &n) && n == INT64_MAX);
  CHECK(ov_parse_int(S("-9223372036854775808"), &n) && n == INT64_MIN);
  CHECK(!ov_parse_int(S("9223372036854775808"), &n));
  CHECK(!ov_parse_int(S("-9223372036854775809"), &n));
  CHECK(!ov_parse_int(S(""), &n));
  CHECK(!ov_parse_int(S("-"), &n));
  CHECK(!ov_parse_int(S(" 1"), &n));
  CHECK(!ov_parse_int(S("1 "), &n));
  CHECK(!ov_parse_int(S("1.5"), &n));
  CHECK(!ov_parse_int(S("0x10"), &n));
  CHECK(ov_parse_float(S("2.5"), &f) && f == 2.5);
  CHECK(ov_parse_float(S("-1e3"), &f) && f == -1000.0);
  CHECK(ov_parse_float(S("3"), &f) && f == 3.0);
  CHECK(!ov_parse_float(S(""), &f));
  CHECK(!ov_parse_float(S("abc"), &f));
  CHECK(!ov_parse_float(S("1.5x"), &f));
  CHECK(!ov_parse_float(S(" 1.5"), &f));
  CHECK(!ov_parse_float(S("inf"), &f));
  CHECK(!ov_parse_float(S("nan"), &f));
  CHECK(!ov_parse_float(S("1e999"), &f));
  CHECK(!ov_parse_float(S("1.5.2"), &f));

  CHECK(STR_IS(ov_show_int(0), "0"));
  CHECK(STR_IS(ov_show_int(-12), "-12"));
  CHECK(STR_IS(ov_show_int(INT64_MIN), "-9223372036854775808"));
  CHECK(STR_IS(ov_show_float(3.0), "3"));
  CHECK(STR_IS(ov_show_float(2.5), "2.5"));
  CHECK(STR_IS(ov_show_float(0.1 + 0.2), "0.30000000000000004"));
  CHECK(STR_IS(ov_show_float(0.1), "0.1"));
  CHECK(STR_IS(ov_show_float(-1.5), "-1.5"));
  CHECK(STR_IS(ov_show_float(0.0), "0"));
  CHECK(STR_IS(ov_show_float(-0.0), "-0"));
  CHECK(STR_IS(ov_show_float(100.0), "100"));
  CHECK(STR_IS(ov_show_float(1e21), "1000000000000000000000"));
  CHECK(STR_IS(ov_show_float(1.5e-7), "0.00000015"));
  CHECK(STR_IS(ov_show_float(123.456), "123.456"));
  CHECK(STR_IS(ov_show_float(1.0 / 0.0), "inf"));
  CHECK(STR_IS(ov_show_float(-1.0 / 0.0), "-inf"));
  CHECK(STR_IS(ov_show_float(0.0 / 0.0), "nan"));
  CHECK(STR_IS(ov_show_bool(true), "true"));
  CHECK(STR_IS(ov_show_bool(false), "false"));
  CHECK(STR_IS(ov_show_unit(), "()"));
  CHECK(STR_IS(ov_show_str(S("a\"b\\c\nd\te\rf\x01")), "\"a\\\"b\\\\c\\nd\\te\\rf\\u{0001}\""));
  CHECK(STR_IS(ov_show_str(S("h\xc3\xa9")), "\"h\xc3\xa9\""));
  CHECK(STR_IS(ov_show_str(S("")), "\"\""));

  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "Point { x: ");
  ov_sb_push(sb, ov_show_int(3));
  for (int i = 0; i < 100; i++) ov_sb_push_cstr(sb, "-");
  ov_str* r = ov_sb_finish(sb);
  CHECK(r->len == 112 && strncmp(r->data, "Point { x: 3---", 15) == 0);
}

static void test_lists(void) {
  ov_list* e = ov_list_empty();
  CHECK(e->len == 0);
  ov_list* xs = ov_list_push(ov_list_push(ov_list_push(e, I(3)), I(1)), I(2));
  CHECK(xs->len == 3 && e->len == 0);
  CHECK(ov_list_get(xs, 0, "t", "h").i == 3);
  CHECK(ov_list_get(xs, 2, "t", "h").i == 2);
  ov_value v;
  CHECK(ov_list_try_get(xs, 1, &v) && v.i == 1);
  CHECK(!ov_list_try_get(xs, 3, &v));
  CHECK(!ov_list_try_get(xs, -1, &v));
  ov_list* z = ov_list_new(2);
  CHECK(z->items[0].i == 0 && z->items[1].i == 0);

  ov_list* c = ov_list_concat(xs, xs);
  CHECK(c->len == 6 && c->items[3].i == 3);
  ov_list* sl = ov_list_slice(xs, 1, 3);
  CHECK(sl->len == 2 && sl->items[0].i == 1);
  CHECK(ov_list_slice(xs, -5, 100)->len == 3);
  CHECK(ov_list_slice(xs, 2, 1)->len == 0);
  CHECK(ov_list_slice(xs, 10, 20)->len == 0);
  CHECK(ov_list_take(xs, 2)->len == 2 && ov_list_take(xs, 2)->items[1].i == 1);
  CHECK(ov_list_take(xs, 99)->len == 3);
  CHECK(ov_list_take(xs, -1)->len == 0);
  CHECK(ov_list_drop(xs, 1)->items[0].i == 1 && ov_list_drop(xs, 1)->len == 2);
  CHECK(ov_list_drop(xs, 99)->len == 0);
  CHECK(ov_list_drop(xs, -1)->len == 3);
  ov_list* rv = ov_list_reverse(xs);
  CHECK(rv->items[0].i == 2 && rv->items[2].i == 3);

  ov_list* sorted = ov_list_sort(xs, ov_cmp_int);
  CHECK(sorted->items[0].i == 1 && sorted->items[1].i == 2 && sorted->items[2].i == 3);
  CHECK(xs->items[0].i == 3); /* original untouched */

  ov_list* fl = ov_list_new(4);
  fl->items[0].f = 2.5; fl->items[1].f = -1.0; fl->items[2].f = 0.0 / 0.0; fl->items[3].f = 0.5;
  ov_list* fs = ov_list_sort(fl, ov_cmp_float);
  CHECK(fs->items[0].f == -1.0 && fs->items[1].f == 0.5 && fs->items[2].f == 2.5 && fs->items[3].f != fs->items[3].f);

  const char* w[] = {"pear", "apple", "fig"};
  ov_list* ss = ov_list_sort(strs(3, w), ov_cmp_str);
  const char* want[] = {"apple", "fig", "pear"};
  CHECK(list_is(ss, 3, want));

  /* stability: sort pairs (key, tag) by key only */
  ov_list* big = ov_list_new(200);
  for (int i = 0; i < 200; i++) big->items[i].i = (int64_t)(i % 5) * 1000 + i;
  int cmp_key(ov_value a, ov_value b);
  ov_list* bs = ov_list_sort(big, cmp_key);
  bool stable = true;
  for (int i = 1; i < 200; i++) {
    if (bs->items[i - 1].i / 1000 > bs->items[i].i / 1000) stable = false;
    if (bs->items[i - 1].i / 1000 == bs->items[i].i / 1000 && bs->items[i - 1].i % 1000 > bs->items[i].i % 1000) stable = false;
  }
  CHECK(stable);
  CHECK(ov_list_sort(e, ov_cmp_int)->len == 0);
}
int cmp_key(ov_value a, ov_value b) { return ov_cmp_int(I(a.i / 1000), I(b.i / 1000)); }

static void test_maps(void) {
  ov_map* m0 = ov_map_new(OV_KEY_STR);
  ov_map* m = ov_map_set(m0, P(S("zeta")), I(1));
  m = ov_map_set(m, P(S("alpha")), I(2));
  m = ov_map_set(m, P(S("mid")), I(3));
  m = ov_map_set(m, P(S("alpha")), I(20)); /* overwrite keeps position */
  CHECK(ov_map_len(m0) == 0 && ov_map_len(m) == 3);
  ov_value v;
  CHECK(ov_map_get(m, P(S("alpha")), &v) && v.i == 20);
  CHECK(!ov_map_get(m, P(S("nope")), &v));
  CHECK(ov_map_contains(m, P(S("zeta"))));
  CHECK(!ov_map_contains(m0, P(S("zeta"))));
  const char* order[] = {"zeta", "alpha", "mid"};
  CHECK(list_is(ov_map_keys(m), 3, order));
  ov_list* vals = ov_map_values(m);
  CHECK(vals->items[0].i == 1 && vals->items[1].i == 20 && vals->items[2].i == 3);
  ov_map* r = ov_map_remove(m, P(S("alpha")));
  const char* order2[] = {"zeta", "mid"};
  CHECK(ov_map_len(r) == 2 && list_is(ov_map_keys(r), 2, order2));
  CHECK(ov_map_len(m) == 3);
  CHECK(ov_map_remove(m, P(S("nope"))) == m);
  CHECK(ov_map_get(r, P(S("mid")), &v) && v.i == 3);
  CHECK(!ov_map_get(r, P(S("alpha")), &v));

  ov_map* im = ov_map_new(OV_KEY_INT);
  for (int64_t i = 0; i < 1000; i++) im = ov_map_set(im, I((i * 7919) % 1009 - 500), I(i));
  CHECK(ov_map_len(im) == 1000);
  CHECK(ov_map_get(im, I((5 * 7919) % 1009 - 500), &v) && v.i == 5);
  CHECK(ov_map_keys(im)->items[3].i == (3 * 7919) % 1009 - 500);
  CHECK(ov_map_get(ov_map_set(ov_map_new(OV_KEY_INT), I(INT64_MIN), I(9)), I(INT64_MIN), &v) && v.i == 9);

  ov_map* bm = ov_map_set(ov_map_set(ov_map_new(OV_KEY_BOOL), (ov_value){.b = true}, I(1)), (ov_value){.b = false}, I(0));
  CHECK(ov_map_len(bm) == 2);
  CHECK(ov_map_get(bm, (ov_value){.b = true}, &v) && v.i == 1);
  CHECK(ov_map_keys(bm)->items[0].b == true);

  ov_set* s = ov_set_new(OV_KEY_STR);
  ov_set* s1 = ov_set_add(ov_set_add(ov_set_add(s, P(S("b"))), P(S("a"))), P(S("b")));
  CHECK(ov_set_len(s) == 0 && ov_set_len(s1) == 2);
  CHECK(ov_set_contains(s1, P(S("a"))) && !ov_set_contains(s1, P(S("c"))));
  const char* so[] = {"b", "a"};
  CHECK(list_is(ov_set_to_list(s1), 2, so));
  ov_set* s2 = ov_set_remove(s1, P(S("b")));
  CHECK(ov_set_len(s2) == 1 && ov_set_len(s1) == 2 && !ov_set_contains(s2, P(S("b"))));
  ov_set* is = ov_set_add(ov_set_new(OV_KEY_INT), I(5));
  CHECK(ov_set_contains(is, I(5)) && ov_set_to_list(is)->items[0].i == 5);
}

static void test_env(void) {
  char* argv[] = {"prog", "one", "two"};
  ov_init(3, argv);
  ov_list* a = ov_args();
  const char* want[] = {"one", "two"};
  CHECK(list_is(a, 2, want));
  int64_t t = ov_now_millis();
  CHECK(t > 1600000000000LL);
  bool lucky = true;
  for (int i = 0; i < 1000; i++) {
    int64_t r = ov_random_int(-3, 3);
    if (r < -3 || r > 3) lucky = false;
  }
  CHECK(lucky);
  CHECK(ov_random_int(7, 7) == 7);
  ov_random_int(INT64_MIN, INT64_MAX);
  ov_print(S("print-ok "));
  ov_println(S("println-ok"));
  ov_flush(); /* smoke: must not crash with pending output */
  ov_flush();
}

int main(void) {
  char* argv[] = {"prog"};
  ov_init(1, argv);
  test_arith();
  test_utf8();
  test_strings();
  test_parse_show();
  test_lists();
  test_maps();
  test_env();
  fflush(stdout);
  printf("%d checks, %d failures\n", checks, failures);
  return failures != 0;
}
