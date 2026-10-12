# Security

## Reporting a vulnerability

Please report security problems privately, not in a public issue. Use
GitHub's private reporting: go to the **Security** tab of this repository and
choose **Report a vulnerability**.

Include what you found, how to reproduce it, and the version (`dict --version`)
or commit you tested. You can expect a reply within a week.

## Scope

`dict` fetches text from Wiktionary, which anyone can edit, and prints it to
your terminal. Reports of particular interest:

- fetched text that can reach the terminal as escape sequences or other
  control characters
- a word or cached entry that can read or write outside the cache directory
  (`dict --cache-dir`)
- anything in `install.sh` that could write somewhere other than the chosen
  prefix

Only the latest commit on `main` is supported.
