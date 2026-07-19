# hooks/commit-msg.sh
#!/bin/sh
set -e
typos "$1"
cog verify --file "$1"
