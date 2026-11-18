#!/bin/sh
jq -c --arg sock "$TREES_SOCK" 'select(.agent_id) | {hookSpecificOutput: {hookEventName: "PreToolUse",
  updatedInput: (.tool_input + {command:
    ("trees-client \($sock | @sh) exec agent-\(.agent_id) -- sh -c " + (.tool_input.command | @sh))})}}'
