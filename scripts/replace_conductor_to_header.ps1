param(
    [switch]$Apply,
    [string]$Root = (Get-Location).Path
)

# Dry-run by default. Use -Apply to write changes.
$ErrorActionPreference = "Stop"

function Test-IsBinaryFile {
    param([string]$Path)
    try {
        $fs = [System.IO.File]::OpenRead($Path)
        $buf = New-Object byte[] 1024
        $read = $fs.Read($buf, 0, $buf.Length)
        $fs.Close()
        for ($i = 0; $i -lt $read; $i++) {
            if ($buf[$i] -eq 0) { return $true }
        }
        return $false
    } catch {
        return $true
    }
}

$selfPath = $MyInvocation.MyCommand.Path

$files = Get-ChildItem -Path $Root -Recurse -File -Force |
    Where-Object {
        $_.FullName -notmatch '\\.git\\' -and
        $_.FullName -notmatch '\\target\\' -and
        $_.FullName -notmatch '\\.codex\\' -and
        $_.FullName -ne $selfPath
    }

$changes = @()

foreach ($f in $files) {
    if (Test-IsBinaryFile -Path $f.FullName) { continue }

    $content = Get-Content -Raw -Path $f.FullName
    if ($null -eq $content) { continue }

    $newContent = $content

    # Replace path segments first to preserve slashes
    $newContent = $newContent -replace 'conductor/', 'header/'

    # Replace whole-word 'conductor' (case-sensitive) everywhere else
    $newContent = $newContent -replace '\bconductor\b', 'header'

    if ($newContent -ne $content) {
        $changes += $f.FullName
        if ($Apply) {
            Set-Content -Path $f.FullName -Value $newContent -Encoding utf8
        }
    }
}

if ($changes.Count -eq 0) {
    Write-Output "No changes detected."
    exit 0
}

Write-Output "Files changed:"
$changes | ForEach-Object { Write-Output $_ }

if (-not $Apply) {
    Write-Output "Dry-run only. Re-run with -Apply to write changes."
}
