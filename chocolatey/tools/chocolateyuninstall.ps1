$shortcut = Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'ihateborders.lnk'

Remove-Item $shortcut -ErrorAction SilentlyContinue
