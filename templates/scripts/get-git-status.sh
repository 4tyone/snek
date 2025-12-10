#!/usr/bin/env bash
#
# get-git-status.sh - Get git status in JSON format
#
# Output: JSON with modified, staged, untracked files

set -euo pipefail

if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "{\"error\": \"Not a git repository\"}"
    exit 1
fi

python3 <<EOF
import subprocess
import json

result = {
    'staged': [],
    'modified': [],
    'untracked': [],
    'branch': '',
    'ahead': 0,
    'behind': 0
}

# Get branch name
try:
    result['branch'] = subprocess.check_output(
        ['git', 'branch', '--show-current'],
        stderr=subprocess.DEVNULL
    ).decode().strip()
except:
    pass

# Get ahead/behind counts
try:
    status = subprocess.check_output(
        ['git', 'status', '--porcelain=v2', '--branch'],
        stderr=subprocess.DEVNULL
    ).decode()
    for line in status.split('\n'):
        if line.startswith('# branch.ab'):
            parts = line.split()
            for part in parts:
                if part.startswith('+'):
                    result['ahead'] = int(part[1:])
                elif part.startswith('-'):
                    result['behind'] = abs(int(part))
except:
    pass

# Get file statuses
try:
    status = subprocess.check_output(
        ['git', 'status', '--porcelain'],
        stderr=subprocess.DEVNULL
    ).decode()
    for line in status.split('\n'):
        if not line:
            continue
        status_code = line[:2]
        filepath = line[3:]

        if status_code[0] in 'MADRC':
            result['staged'].append({'status': status_code[0], 'file': filepath})
        if status_code[1] in 'MD':
            result['modified'].append({'status': status_code[1], 'file': filepath})
        if status_code == '??':
            result['untracked'].append(filepath)
except:
    pass

print(json.dumps(result, indent=2))
EOF
