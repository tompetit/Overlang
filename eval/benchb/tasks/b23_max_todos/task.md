# Cap the list at 20 todos

The list should not grow past 20 todos (done ones count). `POST /add` when 20 are already stored answers `409`
with the body `list full` and stores nothing. Deleting a todo makes room again. Everything else is unchanged.
