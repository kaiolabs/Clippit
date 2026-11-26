# Security

Clippit sees everything you copy and can run commands on your computer, so security reports matter.

**Please don't open a public issue.** Report privately through
[GitHub security advisories](https://github.com/kaiolabs/Clippit/security/advisories/new).
Include what can happen, how to reproduce it and the version (`git rev-parse HEAD`).

You'll get an answer within a week. Fixes go to `master` first and are credited to you unless you
prefer otherwise.

What we consider in scope: anything that leaks clipboard or learned data outside the machine, runs a
command or changes files without the user's confirmation, lets another local user or a web page
control Clippit, or reads files the AI is supposed to skip (keys, passwords).
