# AXONYX — GitHub Linguist PR Guide
# Copyright (c) 2026 Edison Lepiten / AIEONYX

## Overview

This directory contains all files needed to submit AXONYX to GitHub Linguist.
AXONYX is the public display name for the AXON sovereign programming language.
Primary extension: .ax | Secondary: .axn

## Step 1 — Create the grammar repository

Create a new public GitHub repo: github.com/aieonyx/axonyx-grammar

Add one file: AXONYX.tmLanguage.json (copy from docs/linguist/)

Commit message: "feat: AXONYX TextMate grammar for GitHub Linguist"

## Step 2 — Fork GitHub Linguist

Fork: https://github.com/github-linguist/linguist

Clone your fork locally.

## Step 3 — Add grammar as submodule

From inside the linguist fork directory:

  git submodule add https://github.com/aieonyx/axonyx-grammar     vendor/grammars/axonyx-grammar

## Step 4 — Add language entry

Open lib/linguist/languages.yml and add the entry from
docs/linguist/languages_entry.yml (insert alphabetically under A).

## Step 5 — Update vendor/grammars.yml

Add this entry to vendor/grammars.yml:

  - grammar_path: vendor/grammars/axonyx-grammar/AXONYX.tmLanguage.json
    scope: source.axonyx
    url: https://github.com/aieonyx/axonyx-grammar
    branch: main

## Step 6 — Add sample files

Copy the three files from docs/linguist/samples/ into:
  samples/AXONYX/hello.ax
  samples/AXONYX/fibonacci.ax
  samples/AXONYX/sovereign.ax

## Step 7 — Generate language ID

From inside the linguist fork directory:

  ruby script/update-ids

This assigns a unique language_id to AXONYX in languages.yml.
Replace the placeholder 0 with the generated value.

## Step 8 — Run Linguist tests

  bundle exec rake test

All tests must pass before submitting.

## Step 9 — Submit PR

Push your fork and open a PR to github-linguist/linguist.

PR title: Add AXONYX programming language

PR description template:
---
## Add AXONYX

AXONYX is a sovereign systems programming language developed by AIEONYX.

- Homepage: https://github.com/aieonyx/AXON
- Extensions: .ax, .axn
- Grammar: https://github.com/aieonyx/axonyx-grammar
- Sample programs: included in samples/AXONYX/

AXONYX targets seL4 microkernel and POSIX environments.
The language prioritizes security, sovereignty, and zero external dependencies.

Checklist per CONTRIBUTING.md:
- [ ] Language entry added to languages.yml
- [ ] Grammar added as submodule
- [ ] Sample files included
- [ ] language_id generated via script/update-ids
- [ ] All Linguist tests pass
---

## Notes

- language_id: assigned by script/update-ids (replace 0 in languages_entry.yml)
- color: #1A3A5C (sovereign dark blue)
- ace_mode: text (no existing ACE mode for AXONYX)
- The .ax extension is currently unclaimed in Linguist
