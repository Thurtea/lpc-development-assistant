$src = 'E:\Work\AMLP\mud-references'
$dest = 'E:\Work\AMLP\lpc-dev-assistant\mud-references'
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Get-ChildItem -Path $src -File | ForEach-Object {
    $name = $_.Name
    $full = $_.FullName
    Copy-Item -Path $full -Destination $dest -Force
    if ($name -like '*.zip') {
        $out = Join-Path $dest ($_.BaseName)
        New-Item -ItemType Directory -Force -Path $out | Out-Null
        Expand-Archive -Path (Join-Path $dest $name) -DestinationPath $out -Force
    } elseif ($name -like '*.tar.gz' -or $name -like '*.tgz') {
        $base = $_.BaseName
        if ($name -like '*.tar.gz') { $base = $base -replace '\.tar$','' }
        $out = Join-Path $dest $base
        New-Item -ItemType Directory -Force -Path $out | Out-Null
        tar -xzf (Join-Path $dest $name) -C $out
    } else {
        Write-Output "Skipped unknown format: $name"
    }
}
Write-Output "Extraction complete"
