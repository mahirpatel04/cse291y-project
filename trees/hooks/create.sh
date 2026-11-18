#!/bin/sh
jq -r .name | xargs trees-client "$TREES_SOCK" register
