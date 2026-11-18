#!/bin/sh
jq -c --arg v "$TREES_VIRTUAL" --arg s "$TREES_SNAPSHOTS" '
  select(.agent_id) | "\($s)/agent-\(.agent_id)" as $h | .tool_input as $t
  | (if $t.file_path then "file_path" elif $t.path then "path" else empty end) as $f
  | select($t[$f] | startswith($v) and (startswith($h) | not))
  | {hookSpecificOutput: {hookEventName: "PreToolUse",
      updatedInput: ($t + {($f): ($h + ($t[$f] | ltrimstr($v)))})}}'
