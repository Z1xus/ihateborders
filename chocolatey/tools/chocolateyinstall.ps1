$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$shortcut = Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'ihateborders.lnk'

Install-ChocolateyShortcut -ShortcutFilePath $shortcut -TargetPath (Join-Path $toolsDir 'ihateborders.exe')
