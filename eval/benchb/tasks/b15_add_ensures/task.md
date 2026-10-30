# add_todo must insert exactly one row

`add_todo` returns the number of rows the insert changed, and nothing checks it: with a bad database handle it
returns -1 and the caller carries on. Add a postcondition to `add_todo` stating that it inserted exactly one
row (its result is 1), in the same style as the existing precondition. A failed insert must now trap (an
`ensures` violation in Overlang, an assertion error in Python). Normal adding over HTTP is unchanged.
