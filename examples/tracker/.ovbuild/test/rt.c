/* Overlang C runtime implementation. See rt.h for the one-line docs. */
#define _GNU_SOURCE /* memmem */
#include "rt.h"

#include <ctype.h>
#include <gc.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

/* The only global mutable state: argv, the RNG, (and stdout buffering via stdio). */
static int g_argc;
static char** g_argv;
static uint64_t g_rng;
static bool g_rng_seeded;
static bool g_tty; /* stdout is a terminal: println flushes */

void ov_init(int argc, char** argv) {
  GC_INIT();
  g_argc = argc;
  g_argv = argv;
  setvbuf(stdout, NULL, _IOFBF, 1 << 16);
  g_tty = isatty(STDOUT_FILENO);
}

/* ---- traps ---- */

static void json_str(const char* s) {
  putc('"', stderr);
  for (const unsigned char* p = (const unsigned char*)(s ? s : ""); *p; p++) {
    if (*p == '"' || *p == '\\') fprintf(stderr, "\\%c", *p);
    else if (*p == '\n') fputs("\\n", stderr);
    else if (*p == '\t') fputs("\\t", stderr);
    else if (*p == '\r') fputs("\\r", stderr);
    else if (*p < 0x20) fprintf(stderr, "\\u%04x", *p);
    else putc(*p, stderr);
  }
  putc('"', stderr);
}

static void trap_head(const char* kind, const char* fn, const char* hash) {
  fflush(stdout); /* keep fixture output ordered before the trap */
  fputs("{\"kind\": ", stderr); json_str(kind);
  fputs(", \"fn\": ", stderr); json_str(fn);
  fputs(", \"hash\": ", stderr); json_str(hash);
}

void ov_trap(const char* kind, const char* fn, const char* hash, const char* detail_json) {
  trap_head(kind, fn, hash);
  fprintf(stderr, ", \"detail\": %s}\n", detail_json ? detail_json : "null");
  fflush(stderr);
  exit(3);
}

void ov_trap_contract(const char* clause, const char* fn, const char* hash, const char* expr, const char* args_json) {
  trap_head(clause, fn, hash);
  fputs(", \"clause\": ", stderr); json_str(expr);
  fprintf(stderr, ", \"args\": %s}\n", args_json ? args_json : "{}");
  fflush(stderr);
  exit(3);
}

/* ---- arithmetic ---- */

#define OVERFLOW_TRAP(fn, hash) ov_trap("overflow", fn, hash, "\"integer overflow\"")
#define DIV_ZERO_TRAP(fn, hash) ov_trap("div_zero", fn, hash, "\"division by zero\"")

int64_t ov_add(int64_t a, int64_t b, const char* fn, const char* hash) {
  int64_t r;
  if (__builtin_add_overflow(a, b, &r)) OVERFLOW_TRAP(fn, hash);
  return r;
}
int64_t ov_sub(int64_t a, int64_t b, const char* fn, const char* hash) {
  int64_t r;
  if (__builtin_sub_overflow(a, b, &r)) OVERFLOW_TRAP(fn, hash);
  return r;
}
int64_t ov_mul(int64_t a, int64_t b, const char* fn, const char* hash) {
  int64_t r;
  if (__builtin_mul_overflow(a, b, &r)) OVERFLOW_TRAP(fn, hash);
  return r;
}
int64_t ov_neg(int64_t a, const char* fn, const char* hash) {
  if (a == INT64_MIN) OVERFLOW_TRAP(fn, hash);
  return -a;
}
int64_t ov_div(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) DIV_ZERO_TRAP(fn, hash);
  if (b == -1) return ov_neg(a, fn, hash);
  return a / b;
}
int64_t ov_mod(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) DIV_ZERO_TRAP(fn, hash);
  if (b == -1) return 0;
  return a % b;
}
int64_t ov_div_floor(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) DIV_ZERO_TRAP(fn, hash);
  if (b == -1) return ov_neg(a, fn, hash);
  int64_t q = a / b;
  if (a % b != 0 && ((a < 0) != (b < 0))) q--;
  return q;
}
int64_t ov_mod_floor(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) DIV_ZERO_TRAP(fn, hash);
  if (b == -1) return 0;
  int64_t r = a % b;
  if (r != 0 && ((r < 0) != (b < 0))) r += b;
  return r;
}

/* ---- strings ---- */

static void* alloc_bytes(int64_t n) { return GC_MALLOC_ATOMIC((size_t)n); }

static int cp_bytes(unsigned char c) { return c < 0xC0 ? 1 : c < 0xE0 ? 2 : c < 0xF0 ? 3 : 4; }

/* Byte offset of code point index cp (0 <= cp <= cps). */
static int64_t cp_off(const ov_str* s, int64_t cp) {
  if (s->cps == s->len) return cp;
  int64_t off = 0;
  while (cp-- > 0 && off < s->len) {
    off += cp_bytes((unsigned char)s->data[off]);
    if (off > s->len) off = s->len;
  }
  return off;
}

ov_str* ov_str_new(const char* bytes, int64_t len) {
  ov_str* s = GC_MALLOC(sizeof *s);
  char* d = alloc_bytes(len + 1);
  if (len > 0) memcpy(d, bytes, (size_t)len);
  d[len] = 0;
  int64_t cps = 0;
  for (int64_t i = 0; i < len; i++) cps += ((unsigned char)d[i] & 0xC0) != 0x80;
  s->len = len;
  s->cps = cps;
  s->data = d;
  return s;
}

ov_str* ov_str_lit(const char* cstr) { return ov_str_new(cstr, (int64_t)strlen(cstr)); }

ov_str* ov_str_concat(ov_str* a, ov_str* b) {
  char* d = alloc_bytes(a->len + b->len + 1);
  memcpy(d, a->data, (size_t)a->len);
  memcpy(d + a->len, b->data, (size_t)b->len);
  d[a->len + b->len] = 0;
  ov_str* s = GC_MALLOC(sizeof *s);
  s->len = a->len + b->len;
  s->cps = a->cps + b->cps;
  s->data = d;
  return s;
}

bool ov_str_eq(ov_str* a, ov_str* b) { return a->len == b->len && memcmp(a->data, b->data, (size_t)a->len) == 0; }

int ov_str_cmp(ov_str* a, ov_str* b) {
  int64_t n = a->len < b->len ? a->len : b->len;
  int c = memcmp(a->data, b->data, (size_t)n);
  if (c != 0) return c < 0 ? -1 : 1;
  return (a->len > b->len) - (a->len < b->len);
}

int64_t ov_str_len_cps(ov_str* s) { return s->cps; }

ov_list* ov_str_chars(ov_str* s) {
  ov_list* xs = ov_list_new(s->cps);
  int64_t off = 0;
  for (int64_t i = 0; i < s->cps; i++) {
    int64_t n = cp_bytes((unsigned char)s->data[off]);
    if (off + n > s->len) n = s->len - off;
    xs->items[i].p = ov_str_new(s->data + off, n);
    off += n;
  }
  return xs;
}

ov_list* ov_str_split(ov_str* s, ov_str* sep) {
  if (sep->len == 0) return ov_str_chars(s);
  int64_t count = 1;
  for (int64_t off = 0; off <= s->len;) {
    const char* p = off == s->len ? NULL : memmem(s->data + off, (size_t)(s->len - off), sep->data, (size_t)sep->len);
    if (!p) break;
    count++;
    off = (p - s->data) + sep->len;
  }
  ov_list* xs = ov_list_new(count);
  int64_t off = 0;
  for (int64_t i = 0; i < count; i++) {
    const char* p = i == count - 1 ? NULL : memmem(s->data + off, (size_t)(s->len - off), sep->data, (size_t)sep->len);
    int64_t end = p ? p - s->data : s->len;
    xs->items[i].p = ov_str_new(s->data + off, end - off);
    off = end + sep->len;
  }
  return xs;
}

static bool is_ws(char c) { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f'; }

ov_str* ov_str_trim(ov_str* s) {
  int64_t a = 0, b = s->len;
  while (a < b && is_ws(s->data[a])) a++;
  while (b > a && is_ws(s->data[b - 1])) b--;
  return ov_str_new(s->data + a, b - a);
}

static ov_str* map_ascii(ov_str* s, int (*f)(int)) {
  ov_str* r = ov_str_new(s->data, s->len);
  char* d = (char*)r->data;
  for (int64_t i = 0; i < r->len; i++) d[i] = (char)((unsigned char)d[i] < 0x80 ? f((unsigned char)d[i]) : d[i]);
  return r;
}
ov_str* ov_str_to_upper(ov_str* s) { return map_ascii(s, toupper); }
ov_str* ov_str_to_lower(ov_str* s) { return map_ascii(s, tolower); }

bool ov_str_starts_with(ov_str* s, ov_str* p) { return p->len <= s->len && memcmp(s->data, p->data, (size_t)p->len) == 0; }
bool ov_str_ends_with(ov_str* s, ov_str* p) { return p->len <= s->len && memcmp(s->data + s->len - p->len, p->data, (size_t)p->len) == 0; }
bool ov_str_contains(ov_str* s, ov_str* sub) { return sub->len == 0 || memmem(s->data, (size_t)s->len, sub->data, (size_t)sub->len) != NULL; }

ov_str* ov_str_replace(ov_str* s, ov_str* from, ov_str* to) {
  if (from->len == 0) return s;
  ov_sb* sb = ov_sb_new();
  int64_t off = 0;
  for (;;) {
    const char* p = memmem(s->data + off, (size_t)(s->len - off), from->data, (size_t)from->len);
    if (!p) break;
    int64_t at = p - s->data;
    ov_sb_push(sb, ov_str_new(s->data + off, at - off));
    ov_sb_push(sb, to);
    off = at + from->len;
  }
  ov_sb_push(sb, ov_str_new(s->data + off, s->len - off));
  return ov_sb_finish(sb);
}

ov_str* ov_str_substr(ov_str* s, int64_t from, int64_t to) {
  if (from < 0) from = 0;
  if (to > s->cps) to = s->cps;
  if (to <= from) return ov_str_new("", 0);
  int64_t a = cp_off(s, from), b = cp_off(s, to);
  return ov_str_new(s->data + a, b - a);
}

ov_str* ov_str_repeat(ov_str* s, int64_t n) {
  if (n <= 0 || s->len == 0) return ov_str_new("", 0);
  int64_t total;
  if (__builtin_mul_overflow(s->len, n, &total) || total == INT64_MAX) ov_trap("overflow", "ov_str_repeat", "", "\"string too long\"");
  char* d = alloc_bytes(total + 1);
  for (int64_t i = 0; i < n; i++) memcpy(d + i * s->len, s->data, (size_t)s->len);
  d[total] = 0;
  ov_str* r = ov_str_new(d, total);
  return r;
}

ov_str* ov_str_char_at(ov_str* s, int64_t i) {
  if (i < 0 || i >= s->cps) return NULL;
  return ov_str_substr(s, i, i + 1);
}

ov_str* ov_str_join(ov_list* xs, ov_str* sep) {
  ov_sb* sb = ov_sb_new();
  for (int64_t i = 0; i < xs->len; i++) {
    if (i > 0) ov_sb_push(sb, sep);
    ov_sb_push(sb, xs->items[i].p);
  }
  return ov_sb_finish(sb);
}

int64_t ov_str_code_point(ov_str* s, bool* ok) {
  if (s->len == 0) { *ok = false; return 0; }
  *ok = true;
  const unsigned char* p = (const unsigned char*)s->data;
  int n = cp_bytes(p[0]);
  if (n > s->len) n = (int)s->len;
  if (n == 1) return p[0];
  int64_t cp = p[0] & (0xFF >> (n + 1));
  for (int i = 1; i < n; i++) cp = (cp << 6) | (p[i] & 0x3F);
  return cp;
}

ov_str* ov_str_from_code_point(int64_t cp) {
  char b[4];
  int n;
  if (cp < 0 || cp > 0x10FFFF || (cp >= 0xD800 && cp <= 0xDFFF)) cp = 0xFFFD;
  if (cp < 0x80) { b[0] = (char)cp; n = 1; }
  else if (cp < 0x800) { b[0] = (char)(0xC0 | (cp >> 6)); b[1] = (char)(0x80 | (cp & 0x3F)); n = 2; }
  else if (cp < 0x10000) { b[0] = (char)(0xE0 | (cp >> 12)); b[1] = (char)(0x80 | ((cp >> 6) & 0x3F)); b[2] = (char)(0x80 | (cp & 0x3F)); n = 3; }
  else { b[0] = (char)(0xF0 | (cp >> 18)); b[1] = (char)(0x80 | ((cp >> 12) & 0x3F)); b[2] = (char)(0x80 | ((cp >> 6) & 0x3F)); b[3] = (char)(0x80 | (cp & 0x3F)); n = 4; }
  return ov_str_new(b, n);
}

bool ov_parse_int(ov_str* s, int64_t* out) {
  const char* p = s->data;
  const char* end = p + s->len;
  bool neg = false;
  if (p < end && (*p == '+' || *p == '-')) neg = *p++ == '-';
  if (p == end) return false;
  int64_t v = 0; /* accumulate negatively so INT64_MIN parses */
  for (; p < end; p++) {
    if (*p < '0' || *p > '9') return false;
    if (__builtin_mul_overflow(v, 10, &v) || __builtin_sub_overflow(v, *p - '0', &v)) return false;
  }
  if (!neg && __builtin_mul_overflow(v, -1, &v)) return false;
  *out = v;
  return true;
}

bool ov_parse_float(ov_str* s, double* out) {
  if (s->len == 0) return false;
  for (int64_t i = 0; i < s->len; i++)
    if (!strchr("0123456789+-.eE", s->data[i]) || s->data[i] == 0) return false;
  char* end;
  double v = strtod(s->data, &end);
  if (end != s->data + s->len || v != v || v - v != 0) return false; /* partial parse, or inf from overflow */
  *out = v;
  return true;
}

ov_str* ov_show_int(int64_t x) {
  char b[32];
  int n = snprintf(b, sizeof b, "%" PRId64, x);
  return ov_str_new(b, n);
}

ov_str* ov_show_float(double x) {
  if (x != x) return ov_str_lit("nan");
  if (x - x != 0) return ov_str_lit(x < 0 ? "-inf" : "inf");
  char e[40];
  for (int p = 0; p <= 16; p++) {
    snprintf(e, sizeof e, "%.*e", p, x);
    if (strtod(e, NULL) == x) break;
  }
  /* e looks like [-]d[.ddd]e[+-]XX; re-render positionally */
  char digits[24], out[400];
  int nd = 0, o = 0;
  const char* q = e;
  if (*q == '-') out[o++] = *q++;
  for (; *q != 'e'; q++) if (*q != '.') digits[nd++] = *q;
  int pos = atoi(q + 1) + 1; /* digits before the decimal point */
  if (pos >= nd) {
    memcpy(out + o, digits, (size_t)nd); o += nd;
    memset(out + o, '0', (size_t)(pos - nd)); o += pos - nd;
  } else if (pos > 0) {
    memcpy(out + o, digits, (size_t)pos); o += pos;
    out[o++] = '.';
    memcpy(out + o, digits + pos, (size_t)(nd - pos)); o += nd - pos;
  } else {
    out[o++] = '0'; out[o++] = '.';
    memset(out + o, '0', (size_t)-pos); o += -pos;
    memcpy(out + o, digits, (size_t)nd); o += nd;
  }
  return ov_str_new(out, o);
}

ov_str* ov_show_bool(bool b) { return ov_str_lit(b ? "true" : "false"); }
ov_str* ov_show_unit(void) { return ov_str_lit("()"); }

ov_str* ov_show_str(ov_str* s) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "\"");
  for (int64_t i = 0; i < s->len; i++) {
    unsigned char c = (unsigned char)s->data[i];
    char b[16];
    switch (c) {
      case '\n': ov_sb_push_cstr(sb, "\\n"); break;
      case '\t': ov_sb_push_cstr(sb, "\\t"); break;
      case '\r': ov_sb_push_cstr(sb, "\\r"); break;
      case '\\': ov_sb_push_cstr(sb, "\\\\"); break;
      case '"': ov_sb_push_cstr(sb, "\\\""); break;
      default:
        if (c < 0x20 || c == 0x7F) { snprintf(b, sizeof b, "\\u{%04X}", c); ov_sb_push_cstr(sb, b); }
        else { b[0] = (char)c; b[1] = 0; ov_sb_push_cstr(sb, b); }
    }
  }
  ov_sb_push_cstr(sb, "\"");
  return ov_sb_finish(sb);
}

/* ---- string builder ---- */

struct ov_sb { char* buf; int64_t len, cap; };

ov_sb* ov_sb_new(void) {
  ov_sb* sb = GC_MALLOC(sizeof *sb);
  sb->cap = 32;
  sb->buf = alloc_bytes(sb->cap);
  return sb;
}

static void sb_add(ov_sb* sb, const char* p, int64_t n) {
  if (sb->len + n > sb->cap) {
    while (sb->len + n > sb->cap) sb->cap *= 2;
    sb->buf = GC_REALLOC(sb->buf, (size_t)sb->cap);
  }
  memcpy(sb->buf + sb->len, p, (size_t)n);
  sb->len += n;
}

void ov_sb_push(ov_sb* sb, ov_str* s) { sb_add(sb, s->data, s->len); }
void ov_sb_push_cstr(ov_sb* sb, const char* s) { sb_add(sb, s, (int64_t)strlen(s)); }
ov_str* ov_sb_finish(ov_sb* sb) { return ov_str_new(sb->buf, sb->len); }

/* ---- lists ---- */

ov_list* ov_list_new(int64_t len) {
  ov_list* xs = GC_MALLOC(sizeof *xs);
  xs->len = len;
  xs->items = GC_MALLOC((size_t)len * sizeof(ov_value)); /* GC_MALLOC zeroes */
  return xs;
}

ov_list* ov_list_empty(void) { return ov_list_new(0); }

ov_value ov_list_get(ov_list* xs, int64_t i, const char* fn, const char* hash) {
  if (i < 0 || i >= xs->len) {
    char d[96];
    snprintf(d, sizeof d, "\"index %" PRId64 " out of range for length %" PRId64 "\"", i, xs->len);
    ov_trap("bounds", fn, hash, d);
  }
  return xs->items[i];
}

bool ov_list_try_get(ov_list* xs, int64_t i, ov_value* out) {
  if (i < 0 || i >= xs->len) return false;
  *out = xs->items[i];
  return true;
}

ov_list* ov_list_push(ov_list* xs, ov_value x) {
  ov_list* r = ov_list_new(xs->len + 1);
  memcpy(r->items, xs->items, (size_t)xs->len * sizeof(ov_value));
  r->items[xs->len] = x;
  return r;
}

ov_list* ov_list_concat(ov_list* a, ov_list* b) {
  ov_list* r = ov_list_new(a->len + b->len);
  memcpy(r->items, a->items, (size_t)a->len * sizeof(ov_value));
  memcpy(r->items + a->len, b->items, (size_t)b->len * sizeof(ov_value));
  return r;
}

ov_list* ov_list_slice(ov_list* xs, int64_t from, int64_t to) {
  if (from < 0) from = 0;
  if (to > xs->len) to = xs->len;
  if (to < from) to = from;
  if (from > xs->len) from = to = xs->len;
  ov_list* r = ov_list_new(to - from);
  memcpy(r->items, xs->items + from, (size_t)(to - from) * sizeof(ov_value));
  return r;
}

ov_list* ov_list_reverse(ov_list* xs) {
  ov_list* r = ov_list_new(xs->len);
  for (int64_t i = 0; i < xs->len; i++) r->items[i] = xs->items[xs->len - 1 - i];
  return r;
}

static void msort(ov_value* a, ov_value* tmp, int64_t n, int (*cmp)(ov_value, ov_value)) {
  if (n < 2) return;
  int64_t m = n / 2;
  msort(a, tmp, m, cmp);
  msort(a + m, tmp, n - m, cmp);
  int64_t i = 0, j = m, k = 0;
  while (i < m && j < n) tmp[k++] = cmp(a[j], a[i]) < 0 ? a[j++] : a[i++]; /* take left on ties: stable */
  while (i < m) tmp[k++] = a[i++];
  while (j < n) tmp[k++] = a[j++];
  memcpy(a, tmp, (size_t)n * sizeof(ov_value));
}

ov_list* ov_list_sort(ov_list* xs, int (*cmp)(ov_value, ov_value)) {
  ov_list* r = ov_list_new(xs->len);
  memcpy(r->items, xs->items, (size_t)xs->len * sizeof(ov_value));
  ov_value* tmp = GC_MALLOC((size_t)xs->len * sizeof(ov_value));
  msort(r->items, tmp, xs->len, cmp);
  return r;
}

ov_list* ov_list_take(ov_list* xs, int64_t n) { return ov_list_slice(xs, 0, n); }
ov_list* ov_list_drop(ov_list* xs, int64_t n) { return ov_list_slice(xs, n, xs->len); }

int ov_cmp_int(ov_value a, ov_value b) { return (a.i > b.i) - (a.i < b.i); }
int ov_cmp_float(ov_value a, ov_value b) {
  bool an = a.f != a.f, bn = b.f != b.f;
  if (an || bn) return an - bn;
  return (a.f > b.f) - (a.f < b.f);
}
int ov_cmp_str(ov_value a, ov_value b) { return ov_str_cmp(a.p, b.p); }

/* ---- maps and sets ---- */

struct ov_map {
  ov_key_kind kind;
  int64_t len;
  ov_value* keys; /* insertion order */
  ov_value* vals;
  int64_t* idx;   /* open-addressing table of entry indexes, -1 empty */
  int64_t cap;    /* power of two, at least 2 * entries */
};
struct ov_set { ov_map* m; };

static uint64_t hash_key(ov_key_kind kind, ov_value k) {
  if (kind == OV_KEY_STR) {
    const ov_str* s = k.p;
    uint64_t h = 1469598103934665603ULL; /* FNV-1a */
    for (int64_t i = 0; i < s->len; i++) h = (h ^ (unsigned char)s->data[i]) * 1099511628211ULL;
    return h;
  }
  uint64_t x = kind == OV_KEY_BOOL ? (uint64_t)k.b : (uint64_t)k.i;
  x += 0x9E3779B97F4A7C15ULL; /* splitmix64 finalizer */
  x = (x ^ (x >> 30)) * 0xBF58476D1CE4E5B9ULL;
  x = (x ^ (x >> 27)) * 0x94D049BB133111EBULL;
  return x ^ (x >> 31);
}

static bool key_eq(ov_key_kind kind, ov_value a, ov_value b) {
  return kind == OV_KEY_STR ? ov_str_eq(a.p, b.p) : kind == OV_KEY_BOOL ? a.b == b.b : a.i == b.i;
}

static int64_t map_find(const ov_map* m, ov_value k) {
  uint64_t mask = (uint64_t)m->cap - 1;
  for (uint64_t h = hash_key(m->kind, k) & mask;; h = (h + 1) & mask) {
    int64_t e = m->idx[h];
    if (e < 0) return -1;
    if (key_eq(m->kind, m->keys[e], k)) return e;
  }
}

/* Empty map with room for n entries. */
static ov_map* map_alloc(ov_key_kind kind, int64_t n) {
  ov_map* m = GC_MALLOC(sizeof *m);
  m->kind = kind;
  m->cap = 8;
  while (m->cap < 2 * n) m->cap *= 2;
  m->keys = GC_MALLOC((size_t)n * sizeof(ov_value));
  m->vals = GC_MALLOC((size_t)n * sizeof(ov_value));
  m->idx = alloc_bytes(m->cap * (int64_t)sizeof(int64_t));
  memset(m->idx, 0xFF, (size_t)m->cap * sizeof(int64_t));
  return m;
}

/* Append an entry known to be absent. */
static void map_append(ov_map* m, ov_value k, ov_value v) {
  uint64_t mask = (uint64_t)m->cap - 1, h = hash_key(m->kind, k) & mask;
  while (m->idx[h] >= 0) h = (h + 1) & mask;
  m->idx[h] = m->len;
  m->keys[m->len] = k;
  m->vals[m->len] = v;
  m->len++;
}

ov_map* ov_map_new(ov_key_kind kind) { return map_alloc(kind, 0); }

ov_map* ov_map_set(ov_map* m, ov_value k, ov_value v) {
  int64_t at = map_find(m, k);
  ov_map* r = map_alloc(m->kind, m->len + (at < 0));
  for (int64_t i = 0; i < m->len; i++) map_append(r, m->keys[i], i == at ? v : m->vals[i]);
  if (at < 0) map_append(r, k, v);
  return r;
}

bool ov_map_get(ov_map* m, ov_value k, ov_value* out) {
  int64_t at = map_find(m, k);
  if (at < 0) return false;
  *out = m->vals[at];
  return true;
}

ov_map* ov_map_remove(ov_map* m, ov_value k) {
  int64_t at = map_find(m, k);
  if (at < 0) return m;
  ov_map* r = map_alloc(m->kind, m->len - 1);
  for (int64_t i = 0; i < m->len; i++) if (i != at) map_append(r, m->keys[i], m->vals[i]);
  return r;
}

bool ov_map_contains(ov_map* m, ov_value k) { return map_find(m, k) >= 0; }
int64_t ov_map_len(ov_map* m) { return m->len; }

static ov_list* list_of(const ov_value* items, int64_t n) {
  ov_list* xs = ov_list_new(n);
  memcpy(xs->items, items, (size_t)n * sizeof(ov_value));
  return xs;
}
ov_list* ov_map_keys(ov_map* m) { return list_of(m->keys, m->len); }
ov_list* ov_map_values(ov_map* m) { return list_of(m->vals, m->len); }

static ov_set* set_wrap(ov_map* m) {
  ov_set* s = GC_MALLOC(sizeof *s);
  s->m = m;
  return s;
}
ov_set* ov_set_new(ov_key_kind kind) { return set_wrap(ov_map_new(kind)); }
ov_set* ov_set_add(ov_set* s, ov_value k) { return map_find(s->m, k) >= 0 ? s : set_wrap(ov_map_set(s->m, k, (ov_value){.i = 0})); }
ov_set* ov_set_remove(ov_set* s, ov_value k) { return map_find(s->m, k) < 0 ? s : set_wrap(ov_map_remove(s->m, k)); }
bool ov_set_contains(ov_set* s, ov_value k) { return map_find(s->m, k) >= 0; }
int64_t ov_set_len(ov_set* s) { return s->m->len; }
ov_list* ov_set_to_list(ov_set* s) { return ov_map_keys(s->m); }

/* ---- IO and environment ---- */

void ov_print(ov_str* s) { fwrite(s->data, 1, (size_t)s->len, stdout); }
void ov_flush(void) { fflush(stdout); }
void ov_println(ov_str* s) {
  ov_print(s);
  putc('\n', stdout);
  if (g_tty) fflush(stdout);
}

ov_str* ov_read_line(bool* ok) {
  ov_sb* sb = ov_sb_new();
  int c;
  bool any = false;
  while ((c = getc(stdin)) != EOF) {
    any = true;
    if (c == '\n') break;
    char ch = (char)c;
    sb_add(sb, &ch, 1);
  }
  *ok = any;
  if (sb->len > 0 && sb->buf[sb->len - 1] == '\r' && c == '\n') sb->len--;
  return ov_sb_finish(sb);
}

ov_str* ov_read_all(void) {
  ov_sb* sb = ov_sb_new();
  char buf[4096];
  size_t n;
  while ((n = fread(buf, 1, sizeof buf, stdin)) > 0) sb_add(sb, buf, (int64_t)n);
  return ov_sb_finish(sb);
}

ov_list* ov_args(void) {
  int64_t n = g_argc > 1 ? g_argc - 1 : 0;
  ov_list* xs = ov_list_new(n);
  for (int64_t i = 0; i < n; i++) xs->items[i].p = ov_str_lit(g_argv[i + 1]);
  return xs;
}

void ov_exit(int64_t code) {
  fflush(stdout);
  exit((int)code);
}

int64_t ov_now_millis(void) {
  struct timespec ts;
  timespec_get(&ts, TIME_UTC);
  return (int64_t)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
}

static uint64_t rng_next(void) { /* splitmix64 */
  uint64_t z = (g_rng += 0x9E3779B97F4A7C15ULL);
  z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
  z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
  return z ^ (z >> 31);
}

int64_t ov_random_int(int64_t lo, int64_t hi) {
  if (!g_rng_seeded) {
    const char* seed = getenv("OV_SEED");
    if (seed && *seed) g_rng = strtoull(seed, NULL, 10);
    else { struct timespec ts; timespec_get(&ts, TIME_UTC); g_rng = (uint64_t)ts.tv_sec * 1000000007ULL ^ (uint64_t)ts.tv_nsec; }
    g_rng_seeded = true;
  }
  if (lo > hi) { int64_t t = lo; lo = hi; hi = t; }
  uint64_t span = (uint64_t)hi - (uint64_t)lo + 1; /* 0 means the full 2^64 range */
  uint64_t r = rng_next();
  /* shortcut: modulo bias (negligible for small spans), upgrade if uniformity matters */
  return (int64_t)((uint64_t)lo + (span ? r % span : r));
}
