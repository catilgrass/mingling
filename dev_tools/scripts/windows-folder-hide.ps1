# Check `last_check`

$lastCheckFile = Join-Path $PSScriptRoot "last_check"
$currentTime = Get-Date
$timeThreshold = 10

if (Test-Path $lastCheckFile) {
    $lastCheckTime = Get-Content $lastCheckFile | Get-Date
    $timeDiff = ($currentTime - $lastCheckTime).TotalMinutes

    if ($timeDiff -lt $timeThreshold) {
        exit
    }
}

$currentTime.ToString() | Out-File -FilePath $lastCheckFile -Force

# Hide Files

Set-Location -Path (Join-Path $PSScriptRoot "..\..")

Get-ChildItem -Path . -Force -Recurse -ErrorAction SilentlyContinue | Where-Object {
    $_.FullName -notmatch '\\.temp\\' -and $_.FullName -notmatch '\\.git\\'
} | ForEach-Object {
    attrib -h $_.FullName 2>&1 | Out-Null
}

Get-ChildItem -Path . -Force -Recurse -ErrorAction SilentlyContinue | Where-Object {
    $_.Name -match '^\..*' -and $_.FullName -notmatch '\\\.\.$' -and $_.FullName -notmatch '\\\.$'
} | ForEach-Object {
    attrib +h $_.FullName 2>&1 | Out-Null
}

if (Get-Command git -ErrorAction SilentlyContinue) {
    git status --ignored --short | ForEach-Object {
        if ($_ -match '^!!\s+(.+)$') {
            $ignoredPath = $matches[1]
            if ($ignoredPath -notmatch '\.lnk$' -and (Test-Path $ignoredPath)) {
                attrib +h $ignoredPath 2>&1 | Out-Null
            }
        }
    }
}
