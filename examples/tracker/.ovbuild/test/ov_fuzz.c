#include "ov_types.h"
#include <inttypes.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ---- fixed fuzzing support (spec section 7.1) */
#define OVFZ_DEPTH 4

bool ovfz_json;
static jmp_buf* ovfz_jmp;     /* armed while a fuzzed case runs */
static const char* ovfz_kind; /* the last trap caught */
static ov_str* ovfz_detail;   /* its detail, as JSON */
static uint64_t ovfz_rng;
static int64_t ovfz_size;     /* grows from 1 to 16 over the cases: a cap on lengths */

_Noreturn void ovfz_trap(const char* kind, const char* fn, const char* hash, const char* detail_json) {
  if (ovfz_jmp) {
    ovfz_kind = kind;
    ovfz_detail = ov_str_lit(detail_json ? detail_json : "null");
    longjmp(*ovfz_jmp, 1);
  }
  (ov_trap)(kind, fn, hash, detail_json);
}

_Noreturn void ovfz_trap_contract(const char* clause, const char* fn, const char* hash, const char* expr, const char* args_json) {
  if (ovfz_jmp) {
    ovfz_kind = clause;
    ovfz_detail = ovsup_json_str(ov_str_lit(expr));
    longjmp(*ovfz_jmp, 1);
  }
  (ov_trap_contract)(clause, fn, hash, expr, args_json);
}

/* The runtime's trap conditions, checked first so the runtime never traps. */
#define OVFZ_OVERFLOW ovfz_trap("overflow", fn, hash, "\"integer overflow\"")
#define OVFZ_DIV_ZERO ovfz_trap("div_zero", fn, hash, "\"division by zero\"")
int64_t ovfz_add(int64_t a, int64_t b, const char* fn, const char* hash) { int64_t r; if (__builtin_add_overflow(a, b, &r)) OVFZ_OVERFLOW; return r; }
int64_t ovfz_sub(int64_t a, int64_t b, const char* fn, const char* hash) { int64_t r; if (__builtin_sub_overflow(a, b, &r)) OVFZ_OVERFLOW; return r; }
int64_t ovfz_mul(int64_t a, int64_t b, const char* fn, const char* hash) { int64_t r; if (__builtin_mul_overflow(a, b, &r)) OVFZ_OVERFLOW; return r; }
int64_t ovfz_neg(int64_t a, const char* fn, const char* hash) { if (a == INT64_MIN) OVFZ_OVERFLOW; return -a; }
int64_t ovfz_div(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) OVFZ_DIV_ZERO;
  if (b == -1 && a == INT64_MIN) OVFZ_OVERFLOW;
  return (ov_div)(a, b, fn, hash);
}
int64_t ovfz_mod(int64_t a, int64_t b, const char* fn, const char* hash) { if (b == 0) OVFZ_DIV_ZERO; return (ov_mod)(a, b, fn, hash); }
int64_t ovfz_div_floor(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) OVFZ_DIV_ZERO;
  if (b == -1 && a == INT64_MIN) OVFZ_OVERFLOW;
  return (ov_div_floor)(a, b, fn, hash);
}
int64_t ovfz_mod_floor(int64_t a, int64_t b, const char* fn, const char* hash) { if (b == 0) OVFZ_DIV_ZERO; return (ov_mod_floor)(a, b, fn, hash); }
ov_value ovfz_list_get(ov_list* xs, int64_t i, const char* fn, const char* hash) {
  if (i < 0 || i >= xs->len) {
    char d[96];
    snprintf(d, sizeof d, "\"index %" PRId64 " out of range for length %" PRId64 "\"", i, xs->len);
    ovfz_trap("bounds", fn, hash, d);
  }
  return xs->items[i];
}
ov_str* ovfz_str_repeat(ov_str* s, int64_t n) {
  int64_t total = 0;
  if (n > 0 && s->len > 0 && (__builtin_mul_overflow(s->len, n, &total) || total == INT64_MAX)) ovfz_trap("overflow", "ov_str_repeat", "", "\"string too long\"");
  return (ov_str_repeat)(s, n);
}

static uint64_t ovfz_next(void) { /* splitmix64 */
  uint64_t z = (ovfz_rng += 0x9E3779B97F4A7C15ULL);
  z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
  z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
  return z ^ (z >> 31);
}
static uint64_t ovfz_below(uint64_t n) { return ovfz_next() % n; }
static int64_t ovfz_cap(int64_t m) { return ovfz_size < m ? ovfz_size : m; }

/* Scalars. Generators take the nesting depth; shrinkers follow the ovfz_shr protocol. */
static int64_t ovfz_gen_int(int d) {
  static const int64_t edge[] = {0, 1, -1, 2, -2, INT64_MIN, INT64_MAX};
  uint64_t r = ovfz_below(8);
  if (r < 2) return edge[ovfz_below(7)];
  if (r < 6) return (int64_t)ovfz_below(201) - 100;
  int64_t m = (int64_t)(ovfz_next() >> (1 + ovfz_below(63))); /* any magnitude, log-uniform */
  return ovfz_below(2) ? m : -m;
}
static int64_t ovfz_shr_int(int64_t x, int64_t k, int64_t* out) {
  if (x == 0) return 0;
  if (k == 0) { *out = x / 2; return -1; }
  return 1;
}
static double ovfz_gen_float(int d) {
  static const double edge[] = {0.0, 1.0, -1.0, 0.5, -0.5, 1e300, -1e300};
  uint64_t r = ovfz_below(8);
  if (r < 2) return edge[ovfz_below(7)];
  if (r < 6) return ((double)ovfz_below(20001) - 10000.0) / 100.0;
  return ldexp((double)(int64_t)ovfz_next(), -(int)ovfz_below(124));
}
static int64_t ovfz_shr_float(double x, int64_t k, double* out) {
  double c = trunc(x) != x ? trunc(x) : trunc(x / 2);
  if (c == x) return 0;
  if (k == 0) { *out = c; return -1; }
  return 1;
}
static bool ovfz_gen_bool(int d) { return ovfz_below(2) == 1; }
static int64_t ovfz_shr_bool(bool x, int64_t k, bool* out) {
  if (!x) return 0;
  if (k == 0) { *out = false; return -1; }
  return 1;
}
static uint8_t ovfz_gen_unit(int d) { return 0; }
static int64_t ovfz_shr_unit(uint8_t x, int64_t k, uint8_t* out) { return 0; }
static ov_str* ovfz_gen_str(int d) {
  static const char alpha[] = "abcdefghijklmnopqrstuvwxyz0123456789 ";
  static const char* const wide[] = {"\xc3\xa9", "\xe6\x97\xa5"}; /* é, 日 */
  char buf[64];
  int64_t n = (int64_t)ovfz_below((uint64_t)ovfz_cap(12) + 1), len = 0;
  for (int64_t i = 0; i < n; i++) {
    uint64_t r = ovfz_below(sizeof alpha - 1 + 2);
    if (r < sizeof alpha - 1) { buf[len++] = alpha[r]; continue; }
    const char* w = wide[r - (sizeof alpha - 1)];
    memcpy(buf + len, w, strlen(w));
    len += (int64_t)strlen(w);
  }
  return ov_str_new(buf, len);
}
static int64_t ovfz_shr_str(ov_str* x, int64_t k, ov_str** out) {
  if (k < x->cps) { *out = ov_str_concat(ov_str_substr(x, 0, k), ov_str_substr(x, k + 1, x->cps)); return -1; }
  return x->cps;
}
static ov_list* ovfz_list_without(ov_list* x, int64_t k) {
  ov_list* y = ov_list_new(x->len - 1);
  for (int64_t i = 0, j = 0; i < x->len; i++) if (i != k) y->items[j++] = x->items[i];
  return y;
}
static ov_list* ovfz_list_with(ov_list* x, int64_t k, ov_value v) {
  ov_list* y = ov_list_new(x->len);
  for (int64_t i = 0; i < x->len; i++) y->items[i] = x->items[i];
  y->items[k] = v;
  return y;
}

typedef struct ovfz_target {
  const char* name;
  const char* hash;
  const char* skip; /* why it cannot be fuzzed, or NULL */
  int nargs;
  void (*gen)(ov_value* a);
  int64_t (*shr)(ov_value* a, int i, int64_t k, ov_value* out);
  bool (*pre)(ov_value* a);
  void (*call)(ov_value* a);
  ov_str* (*args)(ov_value* a);
} ovfz_target;

/* 0: rejected by requires (or requires trapped), 1: passed, 2: trapped in the call (ovfz_kind, ovfz_detail). */
static int ovfz_run(const ovfz_target* t, ov_value* a) {
  jmp_buf jb;
  volatile int phase = 0;
  ovfz_jmp = &jb;
  if (setjmp(jb)) {
    ovfz_jmp = NULL;
    return phase == 0 ? 0 : 2;
  }
  if (!t->pre(a)) {
    ovfz_jmp = NULL;
    return 0;
  }
  phase = 1;
  t->call(a);
  ovfz_jmp = NULL;
  return 1;
}

int64_t ov_page_count_678cf533_30ac5d54(int64_t);
int64_t ov_page_offset_650fdf0d_bcd456be(int64_t);


// page_count
static void ovfz_gen_0(ov_value* a) {
  a[0] = (ov_value){.i = ovfz_gen_int(0)};
}
static int64_t ovfz_shr_0(ov_value* a, int i, int64_t k, ov_value* out) {
  if (i == 0) {
    int64_t c = 0;
    int64_t m = ovfz_shr_int((a[0]).i, k, &c);
    if (m < 0) *out = (ov_value){.i = c};
    return m;
  }
  return 0;
}
static bool ovfz_pre_0(ov_value* a) {
  int64_t p0 = (a[0]).i; (void)p0;
  bool t1 = (p0 >= INT64_C(0));
  bool t2 = t1;
  if (t2) {
    bool t3 = (p0 <= INT64_C(1000000000));
    t2 = t3;
  }
  if (!t2) return false;
  return true;
}
static void ovfz_call_0(ov_value* a) {
  (void)ov_page_count_678cf533_30ac5d54((a[0]).i);
}
static ov_str* ovfz_args_0(ov_value* a) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "{");
  ov_sb_push_cstr(sb, "\"total\": ");
  ov_sb_push(sb, ovsup_json_str(ov_show_int((a[0]).i)));
  ov_sb_push_cstr(sb, "}");
  return ov_sb_finish(sb);
}

// page_offset
static void ovfz_gen_1(ov_value* a) {
  a[0] = (ov_value){.i = ovfz_gen_int(0)};
}
static int64_t ovfz_shr_1(ov_value* a, int i, int64_t k, ov_value* out) {
  if (i == 0) {
    int64_t c = 0;
    int64_t m = ovfz_shr_int((a[0]).i, k, &c);
    if (m < 0) *out = (ov_value){.i = c};
    return m;
  }
  return 0;
}
static bool ovfz_pre_1(ov_value* a) {
  int64_t p0 = (a[0]).i; (void)p0;
  bool t4 = (p0 >= INT64_C(1));
  bool t5 = t4;
  if (t5) {
    bool t6 = (p0 <= INT64_C(1000000));
    t5 = t6;
  }
  if (!t5) return false;
  return true;
}
static void ovfz_call_1(ov_value* a) {
  (void)ov_page_offset_650fdf0d_bcd456be((a[0]).i);
}
static ov_str* ovfz_args_1(ov_value* a) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "{");
  ov_sb_push_cstr(sb, "\"page\": ");
  ov_sb_push(sb, ovsup_json_str(ov_show_int((a[0]).i)));
  ov_sb_push_cstr(sb, "}");
  return ov_sb_finish(sb);
}

static const ovfz_target ovfz_targets[] = {
  {"page_count", "678cf533ba65e0c1b9068e07b7e1f0626908ef9883f282fd98349e830c870873", NULL, 1, ovfz_gen_0, ovfz_shr_0, ovfz_pre_0, ovfz_call_0, ovfz_args_0},
  {"page_offset", "650fdf0de2a096146995366a8fbd5cf2953dabe35ac5fbfc0202b598324bba70", NULL, 1, ovfz_gen_1, ovfz_shr_1, ovfz_pre_1, ovfz_call_1, ovfz_args_1},
  {"create_issue", "c9275d15243ee94ce20e236ff1fa6b6721c6153eaddb69c23a5c51f532247b19", "has effects !fs", 0, NULL, NULL, NULL, NULL, NULL},
  {"create_project", "2373b13f818ff3bc2a35c24741f008aee346016163cc5e8141474ea7e489e701", "has effects !fs", 0, NULL, NULL, NULL, NULL, NULL},
  {"set_status", "7fbb65d5380a13a8481cbc21945bac036e1026f46196a69043b76367edb925b8", "has effects !fs", 0, NULL, NULL, NULL, NULL, NULL},
  {"search_issues", "573fd6dd20cef705dd845dcfc486b28bec5d50ecb8ba881e363d2bec04e6a0a6", "has effects !fs", 0, NULL, NULL, NULL, NULL, NULL},
  {"create_user", "f316dd39bb9f37330164984a12442626bc2faad1a51cc5e2304873436293bae7", "has effects !fs", 0, NULL, NULL, NULL, NULL, NULL},
  {"create_label", "da024719c092e758180a758bc3ae3d568da3c57898cae3a270c67c656dd989fb", "has effects !fs", 0, NULL, NULL, NULL, NULL, NULL},
  {NULL, NULL, NULL, 0, NULL, NULL, NULL, NULL, NULL},
};

static void ovfz_push_json(ov_sb* o, const char* s) { ov_sb_push(o, ovsup_json_str(ov_str_lit(s))); }

/* `--fuzz <fn> --seed <s> --cases <n>`: fuzz one function and print one JSON line.
 * Returns -1 (run the tests) without --fuzz; --json sets ovfz_json either way. */
int ovfz_main(int argc, char** argv) {
  const char* name = NULL;
  uint64_t seed = 0;
  int64_t cases = 200;
  for (int i = 1; i < argc; i++) {
    if (strcmp(argv[i], "--json") == 0) ovfz_json = true;
    else if (strcmp(argv[i], "--fuzz") == 0 && i + 1 < argc) name = argv[++i];
    else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc) seed = strtoull(argv[++i], NULL, 10);
    else if (strcmp(argv[i], "--cases") == 0 && i + 1 < argc) cases = strtoll(argv[++i], NULL, 10);
  }
  if (!name) return -1;
  const ovfz_target* t = NULL;
  for (const ovfz_target* p = ovfz_targets; p->name; p++)
    if (strcmp(p->name, name) == 0) t = p;
  char buf[96];
  ov_sb* o = ov_sb_new();
  ov_sb_push_cstr(o, "{\"fn\": ");
  ovfz_push_json(o, name);
  ov_sb_push_cstr(o, ", \"hash\": ");
  ovfz_push_json(o, t ? t->hash : "");
  if (!t || t->skip) {
    ov_sb_push_cstr(o, ", \"cases\": 0, \"rejected\": 0, \"status\": \"skipped\", \"reason\": ");
    ovfz_push_json(o, t ? t->skip : "not a fuzz target");
  } else {
    ov_value* a = GC_MALLOC(sizeof(ov_value) * (size_t)(t->nargs + 1));
    int64_t i = 0, rejected = 0;
    int r = 1;
    for (; i < cases; i++) {
      /* Each case has its own stream, so (seed, case) names an input. */
      ovfz_rng = seed * 0x9E3779B97F4A7C15ULL ^ (uint64_t)(i + 1) * 0xD1B54A32D192ED03ULL;
      ovfz_size = 1 + i * 16 / cases;
      t->gen(a);
      r = ovfz_run(t, a);
      if (r == 0) rejected++;
      if (r == 2) break;
    }
    snprintf(buf, sizeof buf, ", \"cases\": %" PRId64 ", \"rejected\": %" PRId64, r == 2 ? i + 1 : i, rejected);
    ov_sb_push_cstr(o, buf);
    if (r != 2) {
      ov_sb_push_cstr(o, ", \"status\": \"ok\"");
    } else {
      const char* kind = ovfz_kind;
      ov_str* detail = ovfz_detail;
      ov_str* orig = t->args(a);
      size_t sz = sizeof(ov_value) * (size_t)(t->nargs + 1);
      ov_value* cur = GC_MALLOC(sz);
      ov_value* trial = GC_MALLOC(sz);
      memcpy(cur, a, sz);
      /* Greedy: take the first candidate of any argument that still fails the same way, then start over. */
      int64_t budget = 10000;
      bool improved = true;
      while (improved && budget > 0) {
        improved = false;
        for (int j = 0; j < t->nargs && !improved; j++) {
          for (int64_t k = 0; budget > 0; k++) {
            ov_value c = {0};
            if (t->shr(cur, j, k, &c) >= 0) break;
            memcpy(trial, cur, sz);
            trial[j] = c;
            budget--;
            if (ovfz_run(t, trial) == 2 && strcmp(ovfz_kind, kind) == 0) {
              cur[j] = c;
              detail = ovfz_detail;
              improved = true;
              break;
            }
          }
        }
      }
      ov_sb_push_cstr(o, ", \"status\": \"fail\", \"failure\": {\"kind\": ");
      ovfz_push_json(o, kind);
      ov_sb_push_cstr(o, ", \"detail\": ");
      ov_sb_push(o, detail);
      ov_sb_push_cstr(o, ", \"args\": ");
      ov_sb_push(o, orig);
      ov_sb_push_cstr(o, ", \"shrunk_args\": ");
      ov_sb_push(o, t->args(cur));
      snprintf(buf, sizeof buf, ", \"seed\": %" PRIu64 ", \"case\": %" PRId64 "}", seed, i);
      ov_sb_push_cstr(o, buf);
    }
  }
  ov_sb_push_cstr(o, "}\n");
  fputs(ov_sb_finish(o)->data, stdout);
  fflush(stdout);
  return 0;
}
