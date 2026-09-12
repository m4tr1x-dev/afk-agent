# Repair the PATH in a shell that was started without it.
#
# Source this when `command -v cargo` fails. Python comes first so that
# `python` resolves to the real interpreter rather than the Windows Store stub
# in WindowsApps, which opens the Store instead of running anything.
#
#     . tools/agent/env.sh

export PATH="/c/Users/msgum/AppData/Local/Programs/Python/Python312/Scripts:/c/Users/msgum/AppData/Local/Programs/Python/Python312:/c/Users/msgum/.cargo/bin:/c/Program Files/dotnet:/c/Users/msgum/.dotnet/tools:/c/Program Files/nodejs:/c/Users/msgum/AppData/Roaming/npm:/c/Program Files/GitHub CLI:$PATH"
