window.SIDEBAR_ITEMS = {"fn":[["delimited","Parse a token stream that starts with `start` and ends with `end`, returning all of the tokens in between. The tokens in between must match `cond`. Succeeds even if there are no tokens between `start` and `end`."],["delimited_some","Parse a token stream that starts with `start` and ends with `end`, returning all of the tokens in between. The tokens in between must match `cond`. There must be at least one token between `start` and `end`."],["eat","Eats the current token if it is `token`."],["eat_any","Eats the current token unconditionally. Fails if there are no tokens."],["eat_if","Eats the token `token` if `cond` holds on the current token."],["eat_slice","Eats the current slice if it is `slice`."],["enclosed","Like `delimited` but keeps the `start` and `end`."],["eof","Succeeds only if the input has reached EOF."],["none","Consumes no tokens. Always succeeds. Equivalent to `take_while(|_| false)`."],["peek","Succeeds if the current token is `token`."],["peek_any","Returns the current token."],["peek_if","Succeeds if `cond` holds for the current token."],["peek_if_copy","Succeeds if `cond` holds for the current token."],["peek_slice","Succeeds if the current slice is `slice`."],["peek_slice_if","Succeeds if the current slice is `slice`."],["skip_any","Skips the current token unconditionally. Fails if there are no tokens."],["skip_while","Skips tokens while `cond` matches."],["take_n","Takes at most `n` tokens."],["take_n_if","Take exactly `n` tokens, ensuring `cond` holds on all `n`."],["take_n_while","Takes at most `n` tokens as long as `cond` holds."],["take_some_while","Consumes tokens while `cond` matches and returns them. Succeeds only if at least one token matched `cond`."],["take_some_while_some_window","Consumes tokens while `cond` matches on a window of tokens of size `n` and returns them. Fails if there aren’t at least `n` tokens or if no tokens match, otherwise returns all of the tokens before the first failure."],["take_some_while_until","Consumes tokens while `cond` matches and the token is not `until`. Succeeds only if at least one token matched `cond`."],["take_some_while_window","Consumes tokens while `cond` matches on a window of tokens of size `n` and returns them. Fails if there no tokens match, otherwise returns all of the tokens before the first failure."],["take_until_slice","Consumes tokens while `cond` matches on a window of tokens of size `n` and returns them. Succeeds even if no tokens match."],["take_while","Consumes tokens while `cond` matches and returns them. Succeeds even if no tokens match."],["take_while_slice","Consumes tokens while `cond` matches on a continously growing slice beginning at a length of `0` and ending when `cond` fails. Returns the slice between `0` and `cond` failing. Errors if no such slice exists."],["take_while_some_window","Consumes tokens while `cond` matches on a window of tokens of size `n` and returns them. Fails if there aren’t at least `n` tokens, otherwise always otherwise always succeeds. If no tokens match, the result will be empty."],["take_while_until","Consumes tokens while `cond` matches and the token is not `until`. Succeeds even if no tokens match."],["take_while_window","Consumes tokens while `cond` matches on a window of tokens of size `n` and returns all of the tokens prior to the first failure to match. For example, given a string of “aaab” and a size 2 window predicate of `window == \"aa\"`, the return value is `\"aa\"` as the first failure to match is at `\"ab\"`."]]};