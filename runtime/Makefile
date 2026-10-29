CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2
# Devbox quirks: a linuxbrew ld can come first on PATH (-B/usr/bin picks the system one), and the system
# libasan.so.6 / libubsan.so.1 sonames are missing from the loader path, so $(B)/.sanlibs symlinks them
# next to the test binaries (rpath $$ORIGIN). Harmless on a normal install.
SAN = -g -fsanitize=address,undefined -fno-sanitize-recover=undefined \
  $(if $(wildcard /usr/bin/ld),-B/usr/bin) -Wl,-rpath,'$$ORIGIN'
B = build

all: librt.a

librt.a: rt.c rt.h
	$(CC) $(CFLAGS) -c rt.c -o rt.o
	ar rcs $@ rt.o

$(B):
	mkdir -p $(B)

$(B)/.sanlibs: | $(B)
	@for l in asan ubsan; do f=$$(realpath $$($(CC) -print-file-name=lib$$l.so)); \
	  so=$$(readelf -d $$f | sed -n 's/.*SONAME.*\[\(.*\)\]/\1/p'); \
	  [ -e /usr/lib64/$$so ] || [ -e /lib64/$$so ] || ln -sf $$f $(B)/$$so; done; touch $@

$(B)/test_rt: rt.c rt.h tests/test_rt.c | $(B)/.sanlibs
	$(CC) $(CFLAGS) $(SAN) rt.c tests/test_rt.c -lgc -o $@

$(B)/trap_main: rt.c rt.h tests/trap_main.c | $(B)/.sanlibs
	$(CC) $(CFLAGS) $(SAN) rt.c tests/trap_main.c -lgc -o $@

test: $(B)/test_rt $(B)/trap_main
	ASAN_OPTIONS=detect_leaks=0 $(B)/test_rt
	ASAN_OPTIONS=detect_leaks=0 sh tests/run_traps.sh $(B)/trap_main

clean:
	rm -rf $(B) rt.o librt.a

.PHONY: all test clean
