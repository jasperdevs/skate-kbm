import { existsSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

patchPowerShellShim();

function patchPowerShellShim() {
  if (process.platform !== "win32") return;

  const prefix = process.env.npm_config_prefix;
  if (!prefix) return;

  const shim = resolve(prefix, "skate-kbm.ps1");
  if (!existsSync(shim)) return;

  writeFileSync(
    shim,
    `#!/usr/bin/env pwsh
$basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent

$exe=""
if ($PSVersionTable.PSVersion -lt "6.0" -or $IsWindows) {
  $exe=".exe"
}

if (Test-Path "$basedir/node$exe") {
  if ($MyInvocation.ExpectingInput) {
    $input | & "$basedir/node$exe" "$basedir/node_modules/skate-kbm/bin/skate-kbm.js" $args
  } else {
    & "$basedir/node$exe" "$basedir/node_modules/skate-kbm/bin/skate-kbm.js" $args
  }
} else {
  if ($MyInvocation.ExpectingInput) {
    $input | & "node$exe" "$basedir/node_modules/skate-kbm/bin/skate-kbm.js" $args
  } else {
    & "node$exe" "$basedir/node_modules/skate-kbm/bin/skate-kbm.js" $args
  }
}

$global:LASTEXITCODE=$LASTEXITCODE
return
`,
  );
}
