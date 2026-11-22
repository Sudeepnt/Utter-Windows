# Installs Utter for the current user from the newest GitHub release:
# downloads UtterSetup-<x64|arm64>.exe, checks it against the release's
# SHA256SUMS.txt, runs it silently, and opens Utter. The installer puts Utter
# in %APPDATA%\utter, adds it to the Start menu and Settings > Apps, and puts
# `utter` on PATH. No administrator rights are needed. Utter's first run
# downloads a speech model.
#
#   powershell -ExecutionPolicy Bypass -Command "iwr https://github.com/Sudeepnt/Utter-Windows/releases/latest/download/install-windows.ps1 -OutFile install-windows.ps1; ./install-windows.ps1"
#
# Uninstall (settings, history and speech models stay):
#
#   powershell -ExecutionPolicy Bypass -File install-windows.ps1 -Uninstall

[CmdletBinding()]
param(
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'
# Windows PowerShell 5.1 downloads many times slower while it draws progress.
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$repo = if ($env:UTTER_REPO) { $env:UTTER_REPO } else { 'Sudeepnt/Utter-Windows' }
$support = Join-Path $env:APPDATA 'utter'
# The installer's AppId (packaging/windows/utter.iss).
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{167AB567-E9F8-4F9B-9F38-61C0C8A17F65}_is1'

if ($Uninstall) {
    $entry = Get-ItemProperty $uninstallKey -ErrorAction SilentlyContinue
    if (-not $entry) {
        Write-Output 'Utter is not installed.'
        exit 0
    }
    $uninstaller = $entry.UninstallString -replace '"', ''
    $process = Start-Process $uninstaller -Wait -PassThru -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART'
    if ($process.ExitCode -ne 0) { throw "The uninstaller failed with exit code $($process.ExitCode)." }
    Write-Output "Removed Utter. Settings and history remain in $support, speech models and logs in $(Join-Path $env:LOCALAPPDATA 'utter')."
    exit 0
}

# A 32-bit PowerShell on 64-bit Windows reports the real machine here.
$machine = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
$label = switch ($machine) {
    'AMD64' { 'x64' }
    'ARM64' { 'arm64' }
    default { throw "Utter supports 64-bit x64 and ARM64 Windows, not $machine." }
}

$base = "https://github.com/$repo/releases/latest/download"
$name = "UtterSetup-$label.exe"
$temp = Join-Path ([IO.Path]::GetTempPath()) ("utter-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $temp | Out-Null
try {
    $sums = Join-Path $temp 'SHA256SUMS.txt'
    $setup = Join-Path $temp $name
    Write-Output "Downloading $name from github.com/$repo..."
    try {
        Invoke-WebRequest -Uri "$base/SHA256SUMS.txt" -OutFile $sums -UseBasicParsing -TimeoutSec 60
        Invoke-WebRequest -Uri "$base/$name" -OutFile $setup -UseBasicParsing -TimeoutSec 900
    } catch {
        if ($_.Exception.Response -and [int]$_.Exception.Response.StatusCode -eq 404) {
            throw "The newest Utter release has no $name. See https://github.com/$repo/releases."
        }
        throw
    }

    $line = Get-Content $sums | Where-Object { $_ -match "^[0-9a-f]{64}\s+\*?$([regex]::Escape($name))$" } | Select-Object -First 1
    if (-not $line) { throw "SHA256SUMS.txt has no entry for $name." }
    $expected = ($line -split '\s+')[0]
    $actual = (Get-FileHash -Algorithm SHA256 $setup).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw "$name failed checksum verification." }

    Write-Output 'Installing Utter...'
    $process = Start-Process $setup -Wait -PassThru -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART'
    if ($process.ExitCode -ne 0) { throw "The installer failed with exit code $($process.ExitCode)." }
} finally {
    Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}

$version = (Get-Content (Join-Path $support 'current-version') -Raw).Trim()
$executable = Join-Path $support "versions\$version\utter.exe"
if (-not (Test-Path $executable)) { throw "The installer finished but $executable is missing." }

Write-Output "Installed Utter $version."
Write-Output "  App:      Start menu > Utter"
Write-Output "  Terminal: utter (open a new terminal first)"
Write-Output "  Remove:   Settings > Apps, or install-windows.ps1 -Uninstall"
# A silent install doesn't open the app, so open it here.
Start-Process $executable -ArgumentList 'app' -WorkingDirectory (Split-Path $executable)
