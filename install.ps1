$ErrorActionPreference = "Stop"

# Default to latest release
$Response = Invoke-RestMethod -Uri "https://api.github.com/repos/enrell/dependencies-graph/releases/latest"
$Version = $Response.tag_name

if (-not $Version) {
    Write-Error "Error: Could not retrieve latest version."
    exit 1
}

Write-Host "Installing depg $Version"

$Target = "x86_64-pc-windows-msvc"
$Url = "https://github.com/enrell/dependencies-graph/releases/download/$Version/depg-$Target.zip"

Write-Host "Downloading from $Url"
$ZipPath = Join-Path $env:TEMP "depg.zip"
Invoke-WebRequest -Uri $Url -OutFile $ZipPath

$ExtractPath = Join-Path $env:TEMP "depg_extract"
if (Test-Path $ExtractPath) { Remove-Item -Recurse -Force $ExtractPath }
Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force

$InstallDir = Join-Path $env:USERPROFILE ".cargo\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$BinPath = Join-Path $InstallDir "depg.exe"
Move-Item -Path (Join-Path $ExtractPath "depg.exe") -Destination $BinPath -Force

Remove-Item $ZipPath
Remove-Item -Recurse -Force $ExtractPath

Write-Host "Successfully installed depg to $BinPath"
Write-Host "Make sure $InstallDir is in your PATH."
