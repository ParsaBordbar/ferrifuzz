# Changelog

## 0.2.1

- Completion inside rule strings: input / output variables, the variable's terms, hedges and
  keywords, read from the `ff.Variable(...)` definitions in the same file.
- Warnings with "did you mean" quick fixes for unknown terms, misspelled variables and
  inputs / outputs on the wrong side of THEN.
- Warning when the selected Python interpreter has no ferrifuzz (or one older than 0.2.0), with
  *Use .venv* and *Select Interpreter* buttons.

## 0.2.0

- Highlighting of rule strings in Python.
- Snippets for the ferrifuzz Python API (`ffmamdani`, `ffsugeno`, `ffit2sys`, ...).
