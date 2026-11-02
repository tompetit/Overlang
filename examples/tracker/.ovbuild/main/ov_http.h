/* Minimal blocking HTTP/1.1 server shim for Overlang `foreign c`. Handles are small positive Ints;
 * errors are negative Ints. Strings returned stay valid forever (Boehm GC). Link with -lgc. */
#ifndef OV_HTTP_H
#define OV_HTTP_H

#include <stdint.h>

/* Listen on 127.0.0.1:port (0 picks a free port); returns a server handle >= 1, or a negative error. */
/* shortcut: loopback only, add a bind-address variant when serving beyond the local machine */
int64_t ov_http_listen(int64_t port);
/* The port the server is actually listening on (useful after listen(0)), or negative for a bad handle. */
int64_t ov_http_port(int64_t server);
/* Block for one connection and read one request (headers plus Content-Length body, at most 1 MB total body);
 * returns a request handle >= 1, or negative (a malformed or oversized request is answered with 400/413 and closed). */
/* shortcut: one request at a time, no keep-alive, no chunked request bodies, upgrade with threads or poll if needed */
int64_t ov_http_accept(int64_t server);
/* The request method, e.g. "GET"; "" for a bad handle. */
const char* ov_http_method(int64_t req);
/* The request path without the query string (not percent-decoded). */
const char* ov_http_path(int64_t req);
/* The raw query string after '?', "" if none. */
const char* ov_http_query(int64_t req);
/* The value of the named header (case-insensitive, surrounding blanks trimmed), "" if absent. */
const char* ov_http_header(int64_t req, const char* name);
/* The request body, "" if none. */
const char* ov_http_body(int64_t req);

/* Percent-decode s, and turn '+' into a space; a malformed %XX is kept literally. */
const char* ov_http_url_decode(const char* s);
/* The decoded value of key in an application/x-www-form-urlencoded body or query string, "" if absent. */
const char* ov_http_form_value(const char* body, const char* key);
/* Escape & < > " ' for safe inclusion in HTML text or attribute values. */
const char* ov_http_html_escape(const char* s);

/* Write a full response (Content-Length, Connection: close), close the socket and free the request slot; returns 0 or negative. */
int64_t ov_http_respond(int64_t req, int64_t status, const char* content_type, const char* body);
/* Close the listening socket and free its handle; a bad handle is ignored. */
void ov_http_close(int64_t server);

#endif
