#!/bin/sh
jq -r '.worktree_path | split("/") | last' | xargs trees-client "$TREES_SOCK" destroy
