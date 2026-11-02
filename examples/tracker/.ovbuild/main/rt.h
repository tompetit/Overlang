/* Overlang C runtime. Every compiled program links against this.
 * All allocation goes through Boehm GC; nothing is ever freed. */
#ifndef OV_RT_H
#define OV_RT_H

#include <stdbool.h>
#include <stdint.h>

/* ---- values and types ---- */

/* A uniform 8-byte slot. Heap objects are referenced through p. Unit is i = 0. */
typedef union ov_value { int64_t i; double f; bool b; void* p; } ov_value;

/* Immutable UTF-8 string. len = bytes, cps = code points, data is NUL-terminated. */
typedef struct ov_str { int64_t len; int64_t cps; const char* data; } ov_str;

/* Immutable once built; builders may fill items before the list escapes. */
typedef struct ov_list { int64_t len; ov_value* items; } ov_list;

typedef struct ov_closure { void* fn; void* env; } ov_closure;

typedef enum ov_key_kind { OV_KEY_INT, OV_KEY_STR, OV_KEY_BOOL } ov_key_kind;

/* Immutable insertion-ordered hash map. */
/* shortcut: O(n) copy on every map_set, upgrade to a HAMT if maps get large */
typedef struct ov_map ov_map;
/* Immutable insertion-ordered set (same representation and shortcut as ov_map). */
typedef struct ov_set ov_set;

/* ---- lifecycle and traps ---- */

/* Initialize the GC, set stdout fully buffered, remember argv for ov_args. */
void ov_init(int argc, char** argv);
/* Flush stdout, print one JSON trap object to stderr, exit 3. detail_json is inserted verbatim (NULL gives null). */
_Noreturn void ov_trap(const char* kind, const char* fn, const char* hash, const char* detail_json);
/* Like ov_trap for a failed contract; kind is clause ("requires"/"ensures"); args_json is verbatim (NULL gives {}). */
_Noreturn void ov_trap_contract(const char* clause, const char* fn, const char* hash, const char* expr, const char* args_json);

/* ---- checked Int arithmetic (trap "overflow", or "div_zero" for zero divisor) ---- */

/* a + b. */
int64_t ov_add(int64_t a, int64_t b, const char* fn, const char* hash);
/* a - b. */
int64_t ov_sub(int64_t a, int64_t b, const char* fn, const char* hash);
/* a * b. */
int64_t ov_mul(int64_t a, int64_t b, const char* fn, const char* hash);
/* a / b truncating toward zero (INT64_MIN / -1 overflows). */
int64_t ov_div(int64_t a, int64_t b, const char* fn, const char* hash);
/* a % b with the sign of the dividend (INT64_MIN % -1 is 0). */
int64_t ov_mod(int64_t a, int64_t b, const char* fn, const char* hash);
/* -a (INT64_MIN overflows). */
int64_t ov_neg(int64_t a, const char* fn, const char* hash);
/* a / b rounded toward negative infinity. */
int64_t ov_div_floor(int64_t a, int64_t b, const char* fn, const char* hash);
/* a - b * div_floor(a, b): result has the sign of the divisor. */
int64_t ov_mod_floor(int64_t a, int64_t b, const char* fn, const char* hash);

/* ---- strings (all results are new immutable strings) ---- */

/* Copy len bytes into a new string (assumed valid UTF-8). */
ov_str* ov_str_new(const char* bytes, int64_t len);
/* New string from a NUL-terminated C string. */
ov_str* ov_str_lit(const char* cstr);
/* a ++ b. */
ov_str* ov_str_concat(ov_str* a, ov_str* b);
/* Byte equality. */
bool ov_str_eq(ov_str* a, ov_str* b);
/* Bytewise comparison: -1, 0 or 1. */
int ov_str_cmp(ov_str* a, ov_str* b);
/* Number of code points. */
int64_t ov_str_len_cps(ov_str* s);
/* List of one-code-point strings. */
ov_list* ov_str_chars(ov_str* s);
/* Split on sep (empty sep splits into chars; adjacent separators yield empty strings, like Python). */
ov_list* ov_str_split(ov_str* s, ov_str* sep);
/* Strip leading and trailing ASCII whitespace. */
ov_str* ov_str_trim(ov_str* s);
/* ASCII upper-casing. shortcut: ASCII only, upgrade when Unicode case mapping is needed */
ov_str* ov_str_to_upper(ov_str* s);
/* ASCII lower-casing. shortcut: ASCII only, upgrade when Unicode case mapping is needed */
ov_str* ov_str_to_lower(ov_str* s);
/* True if s begins with prefix. */
bool ov_str_starts_with(ov_str* s, ov_str* prefix);
/* True if s ends with suffix. */
bool ov_str_ends_with(ov_str* s, ov_str* suffix);
/* True if sub occurs in s (empty sub always does). */
bool ov_str_contains(ov_str* s, ov_str* sub);
/* Replace all non-overlapping occurrences of from with to (empty from returns s). */
ov_str* ov_str_replace(ov_str* s, ov_str* from, ov_str* to);
/* Substring by code point indices [from_cp, to_cp), clamped. */
ov_str* ov_str_substr(ov_str* s, int64_t from_cp, int64_t to_cp);
/* s repeated n times (n <= 0 gives the empty string). */
ov_str* ov_str_repeat(ov_str* s, int64_t n);
/* The code point at index i as a string, or NULL if out of range. */
ov_str* ov_str_char_at(ov_str* s, int64_t i);
/* Join a list of ov_str* with sep. */
ov_str* ov_str_join(ov_list* xs, ov_str* sep);
/* First code point of s; *ok is false (and result 0) if s is empty. */
int64_t ov_str_code_point(ov_str* s, bool* ok);
/* One-code-point string for cp (invalid values give U+FFFD). */
ov_str* ov_str_from_code_point(int64_t cp);

/* Parse optional sign plus decimal digits, whole string, no whitespace; false on bad input or overflow. */
bool ov_parse_int(ov_str* s, int64_t* out);
/* Parse a decimal float (digits, sign, '.', exponent; no inf/nan/hex/whitespace); false on bad input. */
bool ov_parse_float(ov_str* s, double* out);

/* Decimal Int. */
ov_str* ov_show_int(int64_t x);
/* Shortest round-trip decimal, no exponent, no trailing ".0" (3.0 -> "3", 2.5 -> "2.5"); "inf", "-inf", "nan" for non-finite. */
ov_str* ov_show_float(double x);
/* "true" or "false". */
ov_str* ov_show_bool(bool b);
/* Quoted string with \n \t \r \\ \" escapes and \u{XXXX} for other control characters. */
ov_str* ov_show_str(ov_str* s);
/* "()". */
ov_str* ov_show_unit(void);

/* String builder for generated show functions. */
typedef struct ov_sb ov_sb;
/* New empty builder. */
ov_sb* ov_sb_new(void);
/* Append a string. */
void ov_sb_push(ov_sb* sb, ov_str* s);
/* Append a C string. */
void ov_sb_push_cstr(ov_sb* sb, const char* s);
/* Produce the accumulated string (the builder may be reused). */
ov_str* ov_sb_finish(ov_sb* sb);

/* ---- lists ---- */

/* New list of len zeroed slots, to be filled by the caller before it escapes. */
ov_list* ov_list_new(int64_t len);
/* The empty list. */
ov_list* ov_list_empty(void);
/* xs[i], trapping "bounds" if out of range. */
ov_value ov_list_get(ov_list* xs, int64_t i, const char* fn, const char* hash);
/* Store xs[i] in *out and return true, or return false if out of range. */
bool ov_list_try_get(ov_list* xs, int64_t i, ov_value* out);
/* New list with x appended. */
ov_list* ov_list_push(ov_list* xs, ov_value x);
/* a ++ b. */
ov_list* ov_list_concat(ov_list* a, ov_list* b);
/* Elements [from, to), clamped. */
ov_list* ov_list_slice(ov_list* xs, int64_t from, int64_t to);
/* New reversed list. */
ov_list* ov_list_reverse(ov_list* xs);
/* New list sorted by cmp (stable). */
ov_list* ov_list_sort(ov_list* xs, int (*cmp)(ov_value, ov_value));
/* First n elements (clamped). */
ov_list* ov_list_take(ov_list* xs, int64_t n);
/* All but the first n elements (clamped). */
ov_list* ov_list_drop(ov_list* xs, int64_t n);
/* Compare Ints. */
int ov_cmp_int(ov_value a, ov_value b);
/* Compare Floats (NaN sorts last, equal to itself). */
int ov_cmp_float(ov_value a, ov_value b);
/* Compare ov_str* bytewise. */
int ov_cmp_str(ov_value a, ov_value b);

/* ---- maps and sets (keys: Int by value, Str by bytes, Bool by value) ---- */

/* New empty map with the given key kind. */
ov_map* ov_map_new(ov_key_kind kind);
/* New map with k bound to v (existing key keeps its position). */
ov_map* ov_map_set(ov_map* m, ov_value k, ov_value v);
/* Look up k, storing the value in *out; false if absent. */
bool ov_map_get(ov_map* m, ov_value k, ov_value* out);
/* New map without k (the same map if absent). */
ov_map* ov_map_remove(ov_map* m, ov_value k);
/* True if k is present. */
bool ov_map_contains(ov_map* m, ov_value k);
/* Number of entries. */
int64_t ov_map_len(ov_map* m);
/* Keys in insertion order. */
ov_list* ov_map_keys(ov_map* m);
/* Values in key insertion order. */
ov_list* ov_map_values(ov_map* m);

/* New empty set with the given key kind. */
ov_set* ov_set_new(ov_key_kind kind);
/* New set containing k. */
ov_set* ov_set_add(ov_set* s, ov_value k);
/* New set without k. */
ov_set* ov_set_remove(ov_set* s, ov_value k);
/* True if k is present. */
bool ov_set_contains(ov_set* s, ov_value k);
/* Number of elements. */
int64_t ov_set_len(ov_set* s);
/* Elements in insertion order. */
ov_list* ov_set_to_list(ov_set* s);

/* ---- IO and environment ---- */

/* Write s to stdout. */
void ov_print(ov_str* s);
/* Write s and a newline to stdout. */
void ov_println(ov_str* s);
/* Flush buffered stdout (println already flushes when stdout is a terminal). */
void ov_flush(void);
/* Read a line from stdin without the trailing "\n" or "\r\n"; *ok is false at EOF. */
ov_str* ov_read_line(bool* ok);
/* Read all of stdin. */
ov_str* ov_read_all(void);
/* argv[1..] as strings. */
ov_list* ov_args(void);
/* Flush stdout and exit with code. */
_Noreturn void ov_exit(int64_t code);
/* Wall-clock milliseconds since the Unix epoch. */
int64_t ov_now_millis(void);
/* Uniform-ish random Int in [lo, hi]; seeded from env OV_SEED if set, else from time. */
int64_t ov_random_int(int64_t lo, int64_t hi);

#endif
