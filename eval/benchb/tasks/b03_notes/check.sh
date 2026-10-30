#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "call dentist" "notes=ask+about+%26+%3Cb%3Ebridge%3C%2Fb%3E"
page="$(get /)"
expect_has "$page" "<small>ask about &amp; &lt;b&gt;bridge&lt;/b&gt;</small>" "note on page, escaped"
expect_eq "$(grep -o '<small>' <<<"$page" | wc -l)" 1 "one note"
expect_list $'2 [ ] write & ship!\n3 [ ] call dentist'
expect_code 200 /notes "id=2&notes=in+the+evening"
page="$(get /)"
expect_has "$page" "<small>in the evening</small>" "set note"
expect_eq "$(grep -o '<small>' <<<"$page" | wc -l)" 2 "two notes"
post /notes "id=3&notes=" >/dev/null
page="$(get /)"
expect_eq "$(grep -o '<small>' <<<"$page" | wc -l)" 1 "cleared"
expect_code 400 /notes "id=0&notes=x"
expect_code 400 /notes "notes=x"
expect_eq "$(post /notes "id=zz&notes=x")" "bad id" "bad id body"
expect_list $'2 [ ] write & ship!\n3 [ ] call dentist'
bb_restart
expect_has "$(get /)" "<small>in the evening</small>" "persisted"
bb_unit
echo PASS
